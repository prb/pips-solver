// Piece - represents a domino as an ordered pair of Pips

use super::pips::Pips;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pips1: Pips,
    pips2: Pips,
}

impl Piece {
    pub fn new(p1: Pips, p2: Pips) -> Self {
        // Always store in non-descending order
        if p1 <= p2 {
            Piece { pips1: p1, pips2: p2 }
        } else {
            Piece { pips1: p2, pips2: p1 }
        }
    }

    pub fn pips1(&self) -> Pips {
        self.pips1
    }

    pub fn pips2(&self) -> Pips {
        self.pips2
    }

    pub fn is_doubleton(&self) -> bool {
        self.pips1 == self.pips2
    }
}

pub fn remove_one(pieces: Vec<Piece>, piece: Piece) -> Result<Vec<Piece>, String> {
    let mut result = pieces.clone();

    if let Some(pos) = result.iter().position(|&p| p == piece) {
        result.remove(pos);
        Ok(result)
    } else {
        Err(format!("({},{}) was not present in the list of pieces.",
                   piece.pips1().value(), piece.pips2().value()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_ordering() {
        let p1 = Pips::new(1).unwrap();
        let p2 = Pips::new(2).unwrap();

        let piece1 = Piece::new(p1, p2);
        let piece2 = Piece::new(p2, p1);

        assert_eq!(piece1, piece2);
    }

    #[test]
    fn test_doubleton() {
        let p = Pips::new(3).unwrap();
        let piece = Piece::new(p, p);
        assert!(piece.is_doubleton());
    }

    #[test]
    fn test_remove_one() {
        let p1 = Piece::new(Pips::new(1).unwrap(), Pips::new(2).unwrap());
        let p2 = Piece::new(Pips::new(3).unwrap(), Pips::new(4).unwrap());

        let pieces = vec![p1, p1, p2];
        let result = remove_one(pieces, p1).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result, vec![p1, p2]);
    }
}
