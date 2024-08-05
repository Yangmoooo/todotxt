use std::cmp::Ordering;
use std::fmt;
use std::io::Error;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq)]
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
        match s.to_ascii_uppercase().as_str() {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            _ => Ok(Self::O),
        }
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        fn priority_value(priority: &Priority) -> u8 {
            match priority {
                Priority::A => 3,
                Priority::B => 2,
                Priority::C => 1,
                Priority::O => 0,
            }
        }
        priority_value(self).cmp(&priority_value(other))
    }
}
