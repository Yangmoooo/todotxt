pub use chrono::NaiveDate as Date;

pub fn get_date(s: &str) -> Date {
    Date::parse_from_str(s, "%Y-%m-%d").expect("无效的日期格式")
}

pub fn fmt_date(date: &Date) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn today() -> Date {
    chrono::Local::now().date_naive()
}

pub trait DateChecker {
    fn is_over(&self) -> bool;
    fn is_later(&self, other: &Self) -> bool;
}

impl DateChecker for Date {
    fn is_over(&self) -> bool {
        today() > *self
    }

    fn is_later(&self, other: &Self) -> bool {
        *self > *other
    }
}
