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
            let task = tasks::Task::new(
                priority.unwrap_or('O').into(),
                content,
                due_to.map(|s| date::get_date(s.as_str())),
            );
            tasks::add_task(&file_path, task)
        }
        Action::List { mode, keyword, tag } => {
            tasks::list_tasks(&file_path, mode, keyword.as_deref(), tag.as_deref())
        }
        Action::Done { keyword, tag } => {
            tasks::complete_tasks(&file_path, keyword.as_deref(), tag.as_deref())
        }
        Action::Modify { keyword, tag } => {
            tasks::modify_tasks(&file_path, keyword.as_deref(), tag.as_deref())
        }
        Action::Remove { keyword, tag } => {
            tasks::remove_tasks(&file_path, keyword.as_deref(), tag.as_deref())
        }
        Action::Delete { keyword, tag } => {
            tasks::delete_tasks(&file_path, keyword.as_deref(), tag.as_deref())
        }
    }
}

fn get_default_file() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push("todo.txt");
        path
    })
}
