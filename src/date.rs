use chrono::NaiveDate;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Date(NaiveDate);

impl FromStr for Date {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map(Date)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl Date {
    pub fn fmt(&self) -> String {
        self.0.format("%Y-%m-%d").to_string()
    }

    pub fn is_over(&self) -> bool {
        today().0 > self.0
    }
}

pub fn today() -> Date {
    Date(chrono::Local::now().date_naive())
}
