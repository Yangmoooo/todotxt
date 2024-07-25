use std::fmt;
use std::io::Error;
use std::str::FromStr;

#[derive(PartialEq)]
pub enum State {
    Pendding,
    Completed,
    Removed,
}

impl Default for State {
    fn default() -> Self {
        Self::Pendding
    }
}

impl State {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pendding => "",
            Self::Completed => "✓ ",
            Self::Removed => "✗ ",
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pendding => write!(f, ""),
            Self::Completed => write!(f, "✓ "),
            Self::Removed => write!(f, "✗ "),
        }
    }
}

impl FromStr for State {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "✓" => Ok(Self::Completed),
            "✗" => Ok(Self::Removed),
            _ => Ok(Self::Pendding),
        }
    }
}
