use super::pips::Pips;
use std::fmt;

/// Represents a domino piece, stored in non-descending pip order.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Piece {
    left: Pips,
    right: Pips,
}

impl Piece {
    pub fn new(mut a: Pips, mut b: Pips) -> Self {
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        Self { left: a, right: b }
    }

    pub fn left(&self) -> Pips {
        self.left
    }

    pub fn right(&self) -> Pips {
        self.right
    }

    pub fn is_doubleton(&self) -> bool {
        self.left == self.right
    }
}

pub fn remove_one(mut pieces: Vec<Piece>, target: &Piece) -> Result<Vec<Piece>, String> {
    if let Some(index) = pieces.iter().position(|piece| piece == target) {
        pieces.remove(index);
        Ok(pieces)
    } else {
        Err(format!(
            "({},{}) was not present in the list of pieces.",
            target.left, target.right
        ))
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.left, self.right)
    }
}

#[cfg(test)]
mod tests {
    use super::{Piece, remove_one};
    use crate::model::pips::Pips;

    #[test]
    fn pieces_are_sorted() {
        let p1 = Piece::new(Pips::new(2).unwrap(), Pips::new(1).unwrap());
        assert_eq!(p1.left().value(), 1);
        assert_eq!(p1.right().value(), 2);
    }

    #[test]
    fn remove_one_removes_single_occurrence() {
        let p = Piece::new(Pips::new(1).unwrap(), Pips::new(2).unwrap());
        let q = Piece::new(Pips::new(3).unwrap(), Pips::new(4).unwrap());
        let pieces = vec![p.clone(), p.clone(), q.clone()];

        let remaining = remove_one(pieces, &p).unwrap();
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0], p);
        assert_eq!(remaining[1], q);
    }

    #[test]
    fn remove_one_errors_when_missing() {
        let p = Piece::new(Pips::new(1).unwrap(), Pips::new(2).unwrap());
        let pieces = vec![p.clone()];
        let q = Piece::new(Pips::new(3).unwrap(), Pips::new(4).unwrap());
        assert!(remove_one(pieces, &q).is_err());
    }
}
