// Pips - represents the number of pips (0-6) on half of a domino

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pips(u8);

impl Pips {
    pub fn new(value: u8) -> Result<Self, String> {
        if value > 6 {
            Err(format!("Pips value {} is out of range [0..6]", value))
        } else {
            Ok(Pips(value))
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pips() {
        for i in 0..=6 {
            assert!(Pips::new(i).is_ok());
        }
    }

    #[test]
    fn test_invalid_pips() {
        assert!(Pips::new(7).is_err());
        assert!(Pips::new(10).is_err());
    }
}
