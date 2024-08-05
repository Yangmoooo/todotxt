use chrono::NaiveDate;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

#[derive(Clone, Copy)]
pub struct Date(NaiveDate);

impl FromStr for Date {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map(Date)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }
}

impl Date {
    pub fn fmt(&self) -> String {
        self.0.format("%Y-%m-%d").to_string()
    }
}

pub fn today() -> Date {
    Date(chrono::Local::now().date_naive())
}
pub trait DateChecker {
    fn is_over(&self) -> bool;
    fn is_later(&self, other: &Self) -> bool;
    fn is_earlier(&self, other: &Self) -> bool;
}

impl DateChecker for Date {
    fn is_over(&self) -> bool {
        today().0 > self.0
    }

    fn is_later(&self, other: &Self) -> bool {
        self.0 > other.0
    }

    fn is_earlier(&self, other: &Self) -> bool {
        self.0 <= other.0
    }
}
