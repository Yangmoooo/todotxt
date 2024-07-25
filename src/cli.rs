use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::str::FromStr;


bitflags::bitflags! {
    #[derive(Clone)]
    pub struct DisplayMode: u8 {
        const PENDDING = 0b0001;
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
                'p' => mode |= DisplayMode::PENDDING,
                'c' => mode |= DisplayMode::COMPLETED,
                'r' => mode |= DisplayMode::REMOVED,
                _ => return Err(format!("无效的模式字符: {}", c)),
            }
        }
        Ok(mode)
    }
}

#[derive(Subcommand)]
pub enum Action {
    /// 添加一个新任务
    Add {
        /// 任务内容
        content: String,
        /// 优先级
        #[arg(short, long)]
        priority: Option<char>,
        /// 截止日期
        #[arg(short, long)]
        due_to: Option<String>,
    },
    /// 列出任务
    List {
        /// 选择显示模式
        #[arg(short, long, default_value = "p")]
        mode: DisplayMode,
    },
    /// 标记任务为已完成
    Done,
    /// 标记任务为已移除
    Remove,
    /// 删除任务
    Delete,
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
