use super::{assignment::Assignment, piece::Piece, pips::Pips, point::Point};
use std::fmt;

/// Places a polyomino piece at an anchor with a chosen orientation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Placement {
    pub piece: Piece,
    pub anchor: Point,
    pub orientation_index: usize,
    pip_order: Vec<Pips>,
}

impl Placement {
    pub fn new(
        piece: Piece,
        anchor: Point,
        orientation_index: usize,
        pip_order: Vec<Pips>,
    ) -> Self {
        Self {
            piece,
            anchor,
            orientation_index,
            pip_order,
        }
    }

    pub fn orientation_offsets(&self) -> &[(i32, i32)] {
        self.piece.orientations()[self.orientation_index].as_slice()
    }

    pub fn assignments(&self) -> Vec<Assignment> {
        let offsets = self.orientation_offsets();
        self.pip_order
            .iter()
            .zip(offsets.iter())
            .map(|(pip, &(dx, dy))| {
                let x = (self.anchor.x as i32) + dx;
                let y = (self.anchor.y as i32) + dy;
                Assignment::new(*pip, Point::new(x as u32, y as u32))
            })
            .collect()
    }

    pub fn points(&self) -> Vec<Point> {
        self.assignments().into_iter().map(|a| a.point).collect()
    }
}

impl fmt::Display for Placement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} @ {} orientation {}",
            self.piece, self.anchor, self.orientation_index
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Placement;
    use crate::model::{piece::Piece, piece::PolyShape, pips::Pips, point::Point};

    #[test]
    fn placement_assignments_cover_all_cells() {
        let piece = Piece::new(PolyShape::I3, vec![Pips::new(0).unwrap(); 3]).unwrap();
        let pip_order = piece.pip_permutations().pop().unwrap();
        let placement = Placement::new(piece, Point::new(0, 0), 0, pip_order);
        let assignments = placement.assignments();
        assert_eq!(assignments.len(), 3);
    }
}
