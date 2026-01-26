use crate::IString;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(IString);

impl Symbol {
    pub fn new(value: &str) -> Self {
        Self(IString::from(value))
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
