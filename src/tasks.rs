use colored::{Color, ColoredString, Colorize};
use std::collections::HashMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Error, ErrorKind, Result, Write};
use std::path::PathBuf;

use crate::cli::DisplayMode;
use crate::date::{self, Date, DateChecker};
use crate::parser;
use crate::priority::Priority;
use crate::state::State;

// #[cfg(windows)]
// const NEWLINE: &str = "\r\n";
// #[cfg(not(windows))]
// const NEWLINE: &str = "\n";

pub struct Task {
    pub state: State,
    pub priority: Priority,
    pub content: String,
    pub created_at: Date,
    pub due_to: Option<Date>,
    pub completed_at: Option<Date>,
    pub tags: Vec<String>,
}

impl Task {
    pub fn new(priority: Priority, content: String, due_to: Option<Date>) -> Self {
        Self {
            state: State::Pendding,
            priority,
            content,
            created_at: date::today(),
            due_to,
            completed_at: None,
            tags: Vec::new(),
        }
    }
}

impl Task {
    fn fmt_due_to(&self) -> Option<ColoredString> {
        let due_to = self.due_to.map(|date| date::fmt_date(&date))?;
        match self.state {
            State::Completed => Some(due_to.color(
                if self.completed_at.unwrap().is_later(&self.due_to.unwrap()) {
                    Color::Magenta
                } else {
                    Color::Green
                },
            )),
            State::Removed => Some(due_to.dimmed()),
            State::Pendding => Some(due_to.color(if self.due_to.unwrap().is_over() {
                Color::Red
            } else {
                Color::Cyan
            })),
        }
    }

    fn fmt_completed_at(&self) -> Option<ColoredString> {
        self.completed_at.map(|date| date::fmt_date(&date).green())
    }
}

impl Task {
    fn match_mode(&self, mode: &DisplayMode) -> bool {
        match self.state {
            State::Pendding => mode.contains(DisplayMode::PENDDING),
            State::Completed => mode.contains(DisplayMode::COMPLETED),
            State::Removed => mode.contains(DisplayMode::REMOVED),
        }
    }

    fn match_keyword(&self, keyword: Option<&str>) -> bool {
        match keyword {
            Some(k) => self.content.contains(k),
            None => true,
        }
    }

    fn match_tag(&self, tag: Option<&str>) -> bool {
        match tag {
            Some(t) => self.tags.contains(&t.to_string()),
            None => true,
        }
    }

    fn stringify(&self) -> String {
        let mut s = format!(
            "{}[{}] {} ({})",
            self.state,
            self.priority,
            self.content,
            self.created_at.format("%Y-%m-%d")
        );
        if let Some(due_to) = self.due_to {
            s.push_str(&format!(" (due:{})", due_to.format("%Y-%m-%d")));
        }
        if let Some(completed_at) = self.completed_at {
            s.push_str(&format!(" ({})", completed_at.format("%Y-%m-%d")));
        }
        s
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.as_str();
        let priority = self.priority.as_str();
        let content = self.content.as_str();
        let created_at = date::fmt_date(&self.created_at);
        if self.state == State::Removed {
            write!(
                f,
                "{}",
                format!("{state}[{priority}] {content} ({created_at})").dimmed()
            )?;
            if let Some(due_to) = self.fmt_due_to() {
                write!(f, " {}", format!("(due:{due_to})").dimmed())?;
            }
        } else {
            write!(
                f,
                "{}[{}] {} ({})",
                state.green(),
                priority.yellow(),
                content,
                created_at.blue()
            )?;
            if let Some(due_to) = self.fmt_due_to() {
                write!(f, " (due:{due_to})")?;
            }
            if let Some(completed_at) = self.fmt_completed_at() {
                write!(f, " ({completed_at})")?;
            }
        }
        Ok(())
    }
}

fn get_tasks(file_path: &PathBuf) -> Result<Vec<Task>> {
    parser::parse_file(file_path).and_then(|tasks| {
        if tasks.is_empty() {
            Err(Error::new(ErrorKind::InvalidInput, "任务清单为空"))
        } else {
            Ok(tasks)
        }
    })
}

//TODO: 按 优先级 列出任务并排序
//TODO: 按 截止日期 列出任务并排序
pub fn list_tasks(file_path: &PathBuf, mode: DisplayMode, keyword: Option<&str>, tag: Option<&str>) -> Result<()> {
    let tasks = get_tasks(file_path)?;
    let mut i = 0;
    let mut writer = BufWriter::new(io::stdout().lock());
    for task in tasks {
        if task.match_mode(&mode) && task.match_keyword(keyword) && task.match_tag(tag) {
            i += 1;
            writeln!(writer, "{:3} {}", i, task)?;
        }
    }
    writer.flush()?;
    Ok(())
}

pub fn add_task(file_path: &PathBuf, task: Task) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;
    writeln!(file, "{}", task.stringify())?;
    Ok(())
}

//TODO: 优化显示逻辑，倒序显示
fn get_map_pendding_tasks(tasks: &[Task], keyword: Option<&str>, tag: Option<&str>) -> Result<HashMap<usize, usize>> {
    let mut id: usize = 0;
    let mut map = HashMap::new();
    let mut writer = BufWriter::new(io::stdout().lock());
    for (row, task) in tasks.iter().enumerate() {
        if task.state == State::Pendding && task.match_keyword(keyword) && task.match_tag(tag) {
            id += 1;
            map.insert(id, row);
            writeln!(writer, "{:3} {}", id, task)?;
        }
    }
    writer.flush()?;
    Ok(map)
}

fn prompt(action: u8) -> Result<()> {
    print!("{} ", "==>".green());
    match action {
        1 => print!("要完成的任务"),
        2 => print!("要移除的任务 "),
        3 => print!("要删除的任务 "),
        _ => return Err(Error::new(ErrorKind::InvalidInput, "无效的操作")),
    }
    println!("(示例: 1 2 3, 或 1-3)");
    print!("{} ", "==>".green());
    io::stdout().flush()?;

    Ok(())
}

//TODO: 优化输入格式，支持 1-3 形式
fn get_input() -> Result<Vec<usize>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let ids: Vec<usize> = input
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if ids.is_empty() {
        eprintln!("未选择任务");
    }

    Ok(ids)
}

fn write_tasks(file_path: &PathBuf, tasks: Vec<Task>) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    let mut writer = BufWriter::new(file);

    for task in tasks {
        writeln!(writer, "{}", task.stringify())?;
    }
    writer.flush()?;

    Ok(())
}

pub fn complete_tasks(file_path: &PathBuf, keyword: Option<&str>, tag: Option<&str>) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    let id2row = get_map_pendding_tasks(&tasks, keyword, tag)?;
    prompt(1)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        match id2row.get(&id) {
            Some(row) => {
                tasks[*row].state = State::Completed;
                tasks[*row].completed_at = Some(date::today());
            }
            None => eprintln!("无效的任务编号: {}", id),
        }
    }

    write_tasks(file_path, tasks)?;
    Ok(())
}

pub fn remove_tasks(file_path: &PathBuf, keyword: Option<&str>, tag: Option<&str>) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    let id2row = get_map_pendding_tasks(&tasks, keyword, tag)?;
    prompt(2)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        match id2row.get(&id) {
            Some(row) => tasks[*row].state = State::Removed,
            None => eprintln!("无效的任务编号: {}", id),
        }
    }

    write_tasks(file_path, tasks)?;
    Ok(())
}

pub fn delete_tasks(file_path: &PathBuf, keyword: Option<&str>, tag: Option<&str>) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    let id2row = get_map_pendding_tasks(&tasks, keyword, tag)?;
    prompt(3)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        match id2row.get(&id) {
            Some(row) => _ = tasks.remove(*row),
            None => eprintln!("无效的任务编号: {}", id),
        }
    }

    write_tasks(file_path, tasks)?;
    Ok(())
}
