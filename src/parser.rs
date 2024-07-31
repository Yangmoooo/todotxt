use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Result};
use std::path::PathBuf;

use crate::date;
use crate::priority::Priority;
use crate::state::State;
use crate::tasks::Task;

fn parse_line(line: &str, regexes: &[&Regex]) -> Result<Task> {
    let caps = regexes[0]
        .captures(line)
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "无效的任务格式"))?;

    let state: State = match line.chars().next() {
        Some('✓') => State::Completed,
        Some('✗') => State::Removed,
        _ => State::Pending,
    };
    let priority: Priority = caps[1].parse()?;
    let content = caps[2].to_string();
    let created_at = date::get_date(&caps[3]);

    let due_to = if state == State::Completed && caps.get(5).is_none() {
        None // 当任务已完成时，第四个捕获组必是完成日期
    } else {
        caps.get(4).map(|s| date::get_date(s.as_str()))
    };

    let completed_at = if state == State::Completed {
        caps.get(if caps.get(5).is_some() { 5 } else { 4 })
            .map(|s| date::get_date(s.as_str()))
    } else {
        None
    };

    let tags = parse_tag(line, regexes[1]);

    Ok(Task {
        state,
        priority,
        content,
        created_at,
        due_to,
        completed_at,
        tags,
    })
}

fn parse_tag(content: &str, re: &Regex) -> Vec<String> {
    re.captures_iter(content)
        .filter_map(|caps| caps.get(1).map(|tag| tag.as_str().to_string()))
        .collect()
}

pub fn parse_file(file_path: &PathBuf) -> Result<Vec<Task>> {
    let re_line = Regex::new(concat!(
        r"\[(.)\] ",
        r"(.+?) ",
        r"\((\d{4}-\d{2}-\d{2})\)",
        r"(?: \(due:(\d{4}-\d{2}-\d{2})\))?",
        r"(?: \((\d{4}-\d{2}-\d{2})\))?",
    ))
    .unwrap();
    let re_tag = Regex::new(r"(?:\s|^)#(\w+)(?:\s|$)").unwrap();

    let regexes = [&re_line, &re_tag];
    let reader = BufReader::new(File::open(file_path)?);
    let tasks = reader
        .lines()
        .filter_map(|line| line.ok().filter(|line| !line.is_empty()))
        .map(|line| parse_line(&line, &regexes))
        .collect::<Result<Vec<_>>>()?;

    Ok(tasks)
}
