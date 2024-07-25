use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use regex::Regex;

use crate::date;
use crate::priority::Priority;
use crate::state::State;
use crate::tasks::Task;

pub fn parse_line(line: &str, regexes: &[&Regex]) -> Result<Task> {
    let caps = regexes[0]
        .captures(line)
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "无效的任务格式"))?;

    let state: State = match line.chars().next() {
        Some('✓') => State::Completed,
        Some('✗') => State::Removed,
        _ => State::Pendding,
    };
    let priority: Priority = caps[1].parse()?;
    let content = caps[2].to_string();
    let created_at = date::get_date(&caps[3]);

    let due_to = if state == State::Completed && caps.get(5).is_none() {
        None    // 当任务已完成时，第四个捕获组必是完成日期
    } else {
        caps.get(4).map(|s| date::get_date(s.as_str()))
    };

    let completed_at = if state == State::Completed {
        caps.get(if caps.get(5).is_some() { 5 } else { 4 })
            .map(|s| date::get_date(s.as_str()))
    } else {
        None
    };

    let tags = parse_content(line, &regexes[1..]);
    let projects = tags[0].clone();
    let contexts = tags[1].clone();

    Ok(Task {
        state,
        priority,
        content,
        created_at,
        due_to,
        completed_at,
        projects,
        contexts,
    })
}

fn parse_content(content: &str, regexes: &[&Regex]) -> Vec<Vec<String>> {
    regexes.iter().map(|re| parse_tag(content, re)).collect()
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
    let re_project = Regex::new(r"(?:\s|^)#(\w+)(?:\s|$)").unwrap();
    let re_context = Regex::new(r"(?:\s|^)@(\w+)(?:\s|$)").unwrap();

    let regexes = vec![&re_line, &re_project, &re_context];

    let text = fs::read_to_string(file_path)?;
    let tasks = text
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| parse_line(line, &regexes))
        .collect::<Result<Vec<_>>>()?;

    Ok(tasks)
}
