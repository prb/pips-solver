use crate::model::pips::Pips;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece(Pips, Pips);

impl Piece {
    pub fn new(p1: Pips, p2: Pips) -> Self {
        if p1 <= p2 {
            Piece(p1, p2)
        } else {
            Piece(p2, p1)
        }
    }

    pub fn p1(&self) -> Pips {
        self.0
    }

    pub fn p2(&self) -> Pips {
        self.1
    }

    pub fn is_doubleton(&self) -> bool {
        self.0 == self.1
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}

pub fn remove_one(pieces: &[Piece], piece_to_remove: &Piece) -> Result<Vec<Piece>, String> {
    if let Some(pos) = pieces.iter().position(|p| p == piece_to_remove) {
        let mut new_pieces = pieces.to_vec();
        new_pieces.remove(pos);
        Ok(new_pieces)
    } else {
        Err(format!(
            "{} was not present in the list of pieces.",
            piece_to_remove
        ))
    }
}
