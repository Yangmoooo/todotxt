mod cli;
mod date;
mod parser;
mod priority;
mod state;
mod tasks;

use clap::Parser;
use std::path::PathBuf;

use crate::cli::{Action, Args};
use crate::tasks::Task;

fn get_default_file() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push("todo.txt");
        path
    })
}

fn main() -> anyhow::Result<()> {
    let Args { action, file } = Args::parse();

    let file_path = file
        .or_else(get_default_file)
        .ok_or(anyhow::anyhow!("未能找到默认的任务清单文件"))?;

    match action {
        Action::Add {
            content,
            priority,
            due_to,
        } => {
            let task = Task::new(
                priority.unwrap_or('O').into(),
                content,
                due_to.map(|s| date::get_date(s.as_str())),
            );
            tasks::add_task(&file_path, task)
        }
        Action::List { mode } => tasks::list_tasks(&file_path, mode),
        Action::Done => tasks::complete_tasks(&file_path),
        Action::Remove => tasks::remove_tasks(&file_path),
        Action::Delete => tasks::delete_tasks(&file_path),
    }?;

    Ok(())
}
