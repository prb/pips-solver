use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pips(u8);

impl Pips {
    pub fn new(value: u8) -> Result<Self, String> {
        if value <= 6 {
            Ok(Pips(value))
        } else {
            Err(format!("Invalid pips value: {}", value))
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for Pips {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
