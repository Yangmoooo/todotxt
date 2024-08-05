mod cli;
mod date;
mod parser;
mod priority;
mod state;
mod tasks;

use clap::Parser;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

use cli::{Action, Args};
use tasks::Task;

pub fn config() -> (Action, Result<PathBuf, Error>) {
    let args = Args::parse();
    let file = args.file.or_else(get_default_file);
    (
        args.action,
        file.ok_or(Error::new(ErrorKind::InvalidInput, "未指定任务清单文件")),
    )
}

pub fn run(action: Action, file_path: PathBuf) -> Result<(), Error> {
    match action {
        Action::Add {
            content,
            priority,
            due_to,
        } => {
            let task = Task::new(priority.unwrap_or_default(), content, due_to);
            tasks::add_task(&file_path, task)
        }
        Action::List { mode, conf } => tasks::list_tasks(&file_path, mode, &conf),
        Action::Done { conf } => tasks::complete_tasks(&file_path, &conf),
        Action::Modify { conf } => tasks::modify_tasks(&file_path, &conf),
        Action::Remove { conf } => tasks::remove_tasks(&file_path, &conf),
        Action::Delete { conf } => tasks::delete_tasks(&file_path, &conf),
    }
}

fn get_default_file() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push("todo.txt");
        path
    })
}
