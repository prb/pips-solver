use super::{pips::Pips, point::Point};
use std::fmt;

/// Represents assigning a pip value to a specific point on the board.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assignment {
    pub pips: Pips,
    pub point: Point,
}

impl Assignment {
    pub fn new(pips: Pips, point: Point) -> Self {
        Self { pips, point }
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.pips, self.point)
    }
}

#[cfg(test)]
mod tests {
    use super::Assignment;
    use crate::model::{pips::Pips, point::Point};

    #[test]
    fn assignment_stores_values() {
        let p = Assignment::new(Pips::new(3).unwrap(), Point::new(1, 2));
        assert_eq!(p.pips.value(), 3);
        assert_eq!(p.point.x, 1);
        assert_eq!(p.point.y, 2);
    }
}
