use colored::{Color, ColoredString, Colorize};
use std::collections::HashMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, Error, ErrorKind, Result, Write};
use std::path::PathBuf;

use crate::cli::{Action, DisplayMode};
use crate::date::{self, Date, DateChecker};
use crate::parser;
use crate::priority::Priority;
use crate::state::State;

#[cfg(windows)]
const NEWLINE: &str = "\r\n";
#[cfg(not(windows))]
const NEWLINE: &str = "\n";

pub struct Task {
    pub state: State,
    pub priority: Priority,
    pub content: String,
    pub created_at: Date,
    pub due_to: Option<Date>,
    pub completed_at: Option<Date>,
    pub projects: Vec<String>,
    pub contexts: Vec<String>,
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
            projects: Vec::new(),
            contexts: Vec::new(),
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
    fn can_display(&self, mode: &DisplayMode) -> bool {
        match self.state {
            State::Pendding => mode.contains(DisplayMode::PENDDING),
            State::Completed => mode.contains(DisplayMode::COMPLETED),
            State::Removed => mode.contains(DisplayMode::REMOVED),
        }
    }

    fn as_string(&self) -> String {
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

//TODO: 按 标签 列出任务
//TODO: 按 优先级 列出任务并排序
//TODO: 按 截止日期 列出任务并排序
pub fn list_tasks(file_path: &PathBuf, mode: DisplayMode) -> Result<()> {
    let tasks = get_tasks(file_path)?;
    let mut i = 0;
    for task in tasks {
        if task.can_display(&mode) {
            i += 1;
            println!("{:3} {}", i, task);
        }
    }
    Ok(())
}

pub fn add_task(file_path: &PathBuf, task: Task) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;
    writeln!(file, "{}", task.as_string())?;
    Ok(())
}

//TODO: 优化显示逻辑，倒序显示
fn get_map_pendding_tasks(tasks: &[Task]) -> HashMap<usize, usize> {
    let mut id: usize = 0;
    let mut map = HashMap::new();
    for (row, task) in tasks.iter().enumerate() {
        if task.state == State::Pendding {
            id += 1;
            println!("{:3} {}", id, task);
            map.insert(id, row);
        }
    }
    map
}

fn prompt(action: Action) -> Result<()> {
    print!("{} ", "==>".green());
    match action {
        Action::Done => print!("要完成的任务 "),
        Action::Remove => print!("要移除的任务 "),
        Action::Delete => print!("要删除的任务 "),
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
        println!("未选择任务");
    }

    Ok(ids)
}

fn write_tasks(file_path: &PathBuf, tasks: Vec<Task>) -> Result<()> {
    let mut text = String::new();
    for task in tasks {
        text.push_str(&format!("{}{}", task.as_string(), NEWLINE));
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    file.write_all(text.as_bytes())?;

    Ok(())
}

pub fn complete_tasks(file_path: &PathBuf) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    let id2row = get_map_pendding_tasks(&tasks);
    prompt(Action::Done)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        match id2row.get(&id) {
            Some(row) => {
                tasks[*row].state = State::Completed;
                tasks[*row].completed_at = Some(date::today());
            }
            None => println!("无效的任务编号: {}", id),
        }
    }

    write_tasks(file_path, tasks)?;
    Ok(())
}

pub fn remove_tasks(file_path: &PathBuf) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    let id2row = get_map_pendding_tasks(&tasks);
    prompt(Action::Remove)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        match id2row.get(&id) {
            Some(row) => tasks[*row].state = State::Removed,
            None => println!("无效的任务编号: {}", id),
        }
    }

    write_tasks(file_path, tasks)?;
    Ok(())
}

pub fn delete_tasks(file_path: &PathBuf) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    for (i, task) in tasks.iter().enumerate() {
        println!("{:3} {}", i + 1, task);
    }
    prompt(Action::Delete)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        if id == 0 || id > tasks.len() {
            println!("无效的任务编号: {}", id);
        } else {
            tasks.remove(id - 1);
        }
    }

    write_tasks(file_path, tasks)?;
    Ok(())
}
