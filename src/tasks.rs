use colored::{Color, ColoredString, Colorize};
use std::fmt;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::path::PathBuf;

use crate::cli::DisplayMode;
use crate::date::{self, Date, DateChecker};
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

    fn can_display(&self, mode: &DisplayMode) -> bool {
        match self.state {
            State::Pendding => mode.contains(DisplayMode::PENDDING),
            State::Completed => mode.contains(DisplayMode::COMPLETED),
            State::Removed => mode.contains(DisplayMode::REMOVED),
        }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.state.as_str();
        let priority = self.priority.as_str();
        let content = self.content.as_str();
        let created_at = date::fmt_date(&self.created_at);
        if matches!(self.state, State::Removed) {
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

pub fn list_tasks(file_path: &PathBuf, mode: DisplayMode) -> Result<()> {
    let tasks = parser::parse_file(file_path)?;

    if tasks.is_empty() {
        println!("任务清单为空");
    } else {
        let mut i = 0;
        for task in tasks.iter() {
            if task.can_display(&mode) {
                i += 1;
                println!("{:3} {}", i, task);
            }
        }
    }

    Ok(())
}


pub fn add_task(file_path: &PathBuf, task: Task) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;

    write!(
        file,
        "[{}] {} ({})",
        task.priority,
        task.content,
        task.created_at.format("%Y-%m-%d")
    )?;

    match task.due_to {
        Some(due_to) => writeln!(file, " (due:{})", due_to.format("%Y-%m-%d"))?,
        None => writeln!(file)?,
    }

    Ok(())
}

//TODO: 完善其余的任务操作函数
pub fn complete_task(file_path: &PathBuf, id: usize) -> Result<()> {
    let mut file = OpenOptions::new().read(true).open(file_path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;

    let tasks: Vec<_> = text.lines().collect();
    if id == 0 || id > tasks.len() {
        return Err(Error::new(ErrorKind::InvalidInput, "无效的任务 ID"));
    }

    let mut text = String::new();
    for (i, line) in tasks.iter().enumerate() {
        if i + 1 == id {
            text.push_str(&format!(
                "✓ {} ({})",
                line,
                date::today().format("%Y-%m-%d")
            ));
        } else {
            text.push_str(line);
        }
        text.push('\n');
    }
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    file.write_all(text.as_bytes())?;

    Ok(())
}

pub fn remove_task(file_path: &PathBuf, id: usize) -> Result<()> {
    let mut file = OpenOptions::new().read(true).open(file_path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;

    let tasks: Vec<_> = text.lines().collect();
    if id == 0 || id > tasks.len() {
        return Err(Error::new(ErrorKind::InvalidInput, "无效的任务 ID"));
    }

    let mut text = String::new();
    for (i, line) in tasks.iter().enumerate() {
        if i + 1 == id {
            continue;
        }
        text.push_str(line);
        text.push('\n');
    }
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    file.write_all(text.as_bytes())?;

    Ok(())
}

pub fn delete_task(file_path: &PathBuf, id: usize) -> Result<()> {
    todo!()
}
