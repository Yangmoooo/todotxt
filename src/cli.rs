use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;

use crate::date::Date;
use crate::priority::Priority;

bitflags::bitflags! {
    #[derive(Clone)]
    pub struct DisplayMode: u8 {
        const PENDING = 0b0001;
        const COMPLETED = 0b0010;
        const REMOVED = 0b0100;
    }
}

impl FromStr for DisplayMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mode = DisplayMode::empty();
        for c in s.chars() {
            match c {
                'p' => mode |= DisplayMode::PENDING,
                'c' => mode |= DisplayMode::COMPLETED,
                'r' => mode |= DisplayMode::REMOVED,
                _ => return Err(format!("无效的模式字符: {}", c)),
            }
        }
        Ok(mode)
    }
}

#[derive(Parser)]
pub struct TaskConf {
    /// 关键词
    pub keyword: Option<String>,
    /// 标签
    #[arg(short, long)]
    pub tag: Option<String>,
    /// 优先级
    #[arg(short, long)]
    pub priority: Option<Priority>,
    /// 截止日期
    #[arg(short, long)]
    pub due_to: Option<Date>,
}

#[derive(Subcommand)]
pub enum Action {
    /// 添加任务
    Add {
        /// 任务内容
        content: String,
        /// 优先级
        #[arg(short, long)]
        priority: Option<Priority>,
        /// 截止日期
        #[arg(short, long)]
        due_to: Option<Date>,
    },
    /// 列出任务
    List {
        /// 显示模式
        #[arg(short, long, default_value = "p")]
        mode: DisplayMode,
        #[command(flatten)]
        conf: TaskConf,
    },
    /// 完成任务
    Done {
        #[command(flatten)]
        conf: TaskConf,
    },
    /// 修改任务
    Modify {
        #[command(flatten)]
        conf: TaskConf,
    },
    /// 移除任务
    Remove {
        #[command(flatten)]
        conf: TaskConf,
    },
    /// 删除任务
    Delete {
        #[command(flatten)]
        conf: TaskConf,
    },
}

#[derive(Parser)]
#[command(name = "todotxt", version = "0.1.0", author = "Yangmoooo")]
/// 一个基于纯文本的命令行 to-do 清单，受到 todo.txt 的启发
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
    /// 指定任务清单文件
    #[arg(short, long, default_value = "todo.txt")]
    pub file: Option<PathBuf>,
}
