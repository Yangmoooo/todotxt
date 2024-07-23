mod cli;
mod date;
mod parser;
mod priority;
mod tasks;
mod state;

use cli::{Action::*, Args};
use tasks::Task;

use clap::Parser;
use std::path::PathBuf;

fn get_default_file() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push("todo.txt");
        path
    })
}

fn main() -> anyhow::Result<()> {
    let Args { action, file_path } = Args::parse();

    let file_path = file_path
        .or_else(get_default_file)
        .ok_or(anyhow::anyhow!("未能找到默认清单文件"))?;

    match action {
        Add {
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
        List { mode } => tasks::list_tasks(&file_path, mode),
        Done { id } => tasks::complete_task(&file_path, id),
        Remove { id } => tasks::remove_task(&file_path, id),
        Delete { id } => tasks::delete_task(&file_path, id),
    }?;

    Ok(())
}
