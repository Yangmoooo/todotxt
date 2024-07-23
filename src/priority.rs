use std::fmt;
use std::io::Error;
use std::str::FromStr;

pub enum Priority {
    A,
    B,
    C,
    O,
}

impl Default for Priority {
    fn default() -> Self {
        Self::O
    }
}

impl Priority {
    pub fn as_str(&self) -> &str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::O => "O",
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::A => f.write_str("A"),
            Self::B => f.write_str("B"),
            Self::C => f.write_str("C"),
            Self::O => f.write_str("O"),
        }
    }
}

impl From<char> for Priority {
    fn from(c: char) -> Self {
        match c.to_ascii_uppercase() {
            'A' => Self::A,
            'B' => Self::B,
            'C' => Self::C,
            _ => Self::O,
        }
    }
}

impl FromStr for Priority {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            _ => Ok(Self::O),
        }
    }
}
