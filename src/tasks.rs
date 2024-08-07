use colored::{Color, ColoredString, Colorize};
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Error, ErrorKind, Result, Write};
use std::path::PathBuf;

use crate::cli::{DisplayMode, TaskConf};
use crate::date::{self, Date};
use crate::parser;
use crate::priority::Priority;
use crate::state::State;

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
            state: State::Pending,
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
        let due_to = self.due_to.map(|date| date.fmt())?;
        match self.state {
            State::Completed => Some(due_to.color(
                if self.completed_at.unwrap() > self.due_to.unwrap() {
                    Color::Magenta
                } else {
                    Color::Green
                },
            )),
            State::Removed => Some(due_to.dimmed()),
            State::Pending => Some(due_to.color(if self.due_to.unwrap().is_over() {
                Color::Red
            } else {
                Color::Cyan
            })),
        }
    }

    fn fmt_completed_at(&self) -> Option<ColoredString> {
        self.completed_at.map(|date| date.fmt().green())
    }
}

impl Task {
    fn match_mode(&self, mode: &DisplayMode) -> bool {
        match self.state {
            State::Pending => mode.contains(DisplayMode::PENDING),
            State::Completed => mode.contains(DisplayMode::COMPLETED),
            State::Removed => mode.contains(DisplayMode::REMOVED),
        }
    }

    fn match_conf(&self, conf: &TaskConf) -> bool {
        self.contain_keyword(conf.keyword.as_deref())
            && self.contain_tag(conf.tag.as_deref())
            && self.higher_priority(conf.priority)
            && self.before_due_to(conf.due_to)
    }

    fn contain_keyword(&self, keyword: Option<&str>) -> bool {
        match keyword {
            Some(k) => self.content.contains(k),
            None => true,
        }
    }

    fn contain_tag(&self, tag: Option<&str>) -> bool {
        match tag {
            Some(t) => self.tags.contains(&t.to_string()),
            None => true,
        }
    }

    fn higher_priority(&self, priority: Option<Priority>) -> bool {
        match priority {
            Some(p) => self.priority >= p,
            None => true,
        }
    }

    fn before_due_to(&self, due_to: Option<Date>) -> bool {
        match due_to {
            Some(d) if self.due_to.is_some() => self.due_to.unwrap() <= d,
            Some(_) => false,
            None => true,
        }
    }

    fn stringify(&self) -> String {
        let mut s = format!(
            "{}[{}] {} ({})",
            self.state,
            self.priority,
            self.content,
            self.created_at.fmt()
        );
        if let Some(due_to) = self.due_to {
            s.push_str(&format!(" (due:{})", due_to.fmt()));
        }
        if let Some(completed_at) = self.completed_at {
            s.push_str(&format!(" ({})", completed_at.fmt()));
        }
        s
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.as_str();
        let priority = self.priority.as_str();
        let content = self.content.as_str();
        let created_at = self.created_at.fmt();
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

/* 非交互式命令 */

pub fn add_task(file_path: &PathBuf, task: Task) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;
    writeln!(file, "{}", task.stringify())?;
    Ok(())
}

pub fn list_tasks(file_path: &PathBuf, mode: &DisplayMode, conf: &TaskConf) -> Result<()> {
    let mut tasks = get_tasks(file_path)?
        .into_iter()
        .filter(|task| task.match_mode(mode) && task.match_conf(conf))
        .collect::<Vec<_>>();
    if let Some(sort_by) = &conf.sort_by {
        match sort_by.as_str() {
            "p" | "priority" => tasks.sort_by(|a, b| b.priority.cmp(&a.priority)),
            "d" | "due" => tasks.sort_by(|a, b| b.due_to.cmp(&a.due_to)),
            _ => (),
        }
    }

    let mut i = tasks.len() + 1;
    let mut writer = BufWriter::new(io::stdout().lock());
    for task in tasks.iter().rev() {
        i -= 1;
        writeln!(writer, "{:3} {}", i, task)?;
    }
    writer.flush()?;
    Ok(())
}

/* 交互式命令 */

pub fn complete_tasks(file_path: &PathBuf, conf: &TaskConf) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    let id2row = build_map(&tasks, conf)?;
    prompt(1)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        match id2row.get(&id) {
            Some(row) => {
                tasks[*row].state = State::Completed;
                tasks[*row].completed_at = Some(date::today());
            }
            None => eprintln!("{} 无效的任务编号: {}", "==>".red(), id),
        }
    }

    write_tasks(file_path, tasks)?;
    Ok(())
}

pub fn modify_tasks(file_path: &PathBuf, conf: &TaskConf) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    let id2row = build_map(&tasks, conf)?;
    prompt(2)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        match id2row.get(&id) {
            Some(row) => {
                println!("{} 任务 {} 要修改的字段是?", "==>".cyan(), id);
                println!(
                    "{} 优先级 [P]riority, 内容 [C]ontent 或者 截止日期 [D]ue",
                    "==>".cyan()
                );
                prompt_input()?;

                edit_task(tasks.get_mut(*row).expect("获取任务失败"))?;
            }
            None => eprintln!("{} 无效的任务编号: {}", "==>".red(), id),
        }
    }
    write_tasks(file_path, tasks)?;

    Ok(())
}

pub fn remove_tasks(file_path: &PathBuf, conf: &TaskConf) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    let id2row = build_map(&tasks, conf)?;
    prompt(2)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        match id2row.get(&id) {
            Some(row) => tasks[*row].state = State::Removed,
            None => eprintln!("{} 无效的任务编号: {}", "==>".red(), id),
        }
    }

    write_tasks(file_path, tasks)?;
    Ok(())
}

pub fn delete_tasks(file_path: &PathBuf, conf: &TaskConf) -> Result<()> {
    let mut tasks = get_tasks(file_path)?;

    let id2row = build_map(&tasks, conf)?;
    prompt(3)?;

    let selected_ids = get_input()?;
    for id in selected_ids {
        match id2row.get(&id) {
            Some(row) => _ = tasks.remove(*row),
            None => eprintln!("{} 无效的任务编号: {}", "==>".red(), id),
        }
    }

    write_tasks(file_path, tasks)?;
    Ok(())
}

/* 功能函数 */

fn get_tasks(file_path: &PathBuf) -> Result<Vec<Task>> {
    parser::parse_file(file_path).and_then(|tasks| {
        if tasks.is_empty() {
            Err(Error::new(ErrorKind::InvalidInput, "任务清单为空"))
        } else {
            Ok(tasks)
        }
    })
}

fn build_map(tasks: &[Task], conf: &TaskConf) -> Result<HashMap<usize, usize>> {
    let mut sorted_tasks: Vec<(usize, &Task)> = tasks
        .iter()
        .enumerate()
        .filter(|(_, task)| task.state == State::Pending && task.match_conf(conf))
        .collect();
    if let Some(sort_by) = &conf.sort_by {
        match sort_by.as_str() {
            "p" | "priority" => sorted_tasks.sort_by(|a, b| b.1.priority.cmp(&a.1.priority)),
            "d" | "due" => sorted_tasks.sort_by(|a, b| b.1.due_to.cmp(&a.1.due_to)),
            _ => (),
        }
    }

    let mut id = sorted_tasks.len() + 1;
    let mut map = HashMap::new();
    let mut writer = BufWriter::new(io::stdout().lock());
    for (row, task) in sorted_tasks.iter().rev() {
        id -= 1;
        map.insert(id, *row);
        writeln!(writer, "{:3} {}", id, task)?;
    }
    writer.flush()?;
    Ok(map)
}

fn prompt(action: u8) -> Result<()> {
    print!("{} ", "==>".cyan());
    match action {
        1 => print!("要完成的任务"),
        2 => print!("要修改的任务"),
        3 => print!("要移除的任务"),
        4 => print!("要删除的任务"),
        _ => return Err(Error::new(ErrorKind::InvalidInput, "无效的操作")),
    }
    println!(": (示例: 1 3 4)");
    prompt_input()?;

    Ok(())
}

fn prompt_input() -> Result<()> {
    print!("{} ", "==>".green());
    io::stdout().flush()?;
    Ok(())
}

fn get_input() -> Result<Vec<usize>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let ids: Vec<usize> = input
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if ids.is_empty() {
        eprintln!("{} 未输入任务编号", "==>".red());
    }

    Ok(ids)
}

fn edit_task(task: &mut Task) -> Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let field = input.trim();
    match field {
        "P" | "p" => {
            println!("{} 优先级: (A/B/C/O)", "==>".cyan());
            prompt_input()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            task.priority = input.trim().chars().next().unwrap_or('O').into();
        }
        "C" | "c" => {
            println!("{} 内容:", "==>".cyan());
            prompt_input()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().is_empty() {
                eprintln!("{} 未输入内容", "==>".red());
            } else {
                task.content = input.trim().to_string();
                let re_tag = Regex::new(r"(?:\s|^)#(\w+)(?:\s|$)").unwrap();
                task.tags = parser::parse_tags(&task.content, &re_tag);
            }
        }
        "D" | "d" => {
            println!("{} 截止日期: (YYYY-MM-DD)", "==>".cyan());
            prompt_input()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if input.trim().is_empty() {
                task.due_to = None;
            } else {
                // task.due_to = Some(date::get_date(input.trim()));
                task.due_to = Some(input.trim().parse().unwrap());
            }
        }
        _ => eprintln!("{} 无效的字段: {}", "==>".red(), field),
    }

    Ok(())
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
