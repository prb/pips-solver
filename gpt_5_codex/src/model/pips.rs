use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

/// Represents the number of pips on half a domino; guaranteed to be in `[0,6]`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pips(u8);

impl Pips {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 6;

    pub fn new(value: u8) -> Result<Self, String> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Ok(Self(value))
        } else {
            Err(format!(
                "Pips value {} is outside of the allowed range {}-{}.",
                value,
                Self::MIN,
                Self::MAX
            ))
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }
}

impl fmt::Display for Pips {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Pips {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s
            .parse::<u8>()
            .map_err(|err: ParseIntError| err.to_string())?;
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Pips;

    #[test]
    fn creates_valid_pips() {
        for value in Pips::MIN..=Pips::MAX {
            let p = Pips::new(value).unwrap();
            assert_eq!(p.value(), value);
        }
    }

    #[test]
    fn rejects_invalid_pips() {
        assert!(Pips::new(7).is_err());
        assert!(Pips::new(255).is_err());
    }
}
