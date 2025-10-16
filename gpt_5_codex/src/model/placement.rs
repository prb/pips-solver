use super::{assignment::Assignment, direction::Direction, piece::Piece, point::Point};
use std::collections::HashSet;
use std::fmt;

/// Places a piece at a point using a specific direction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Placement {
    pub piece: Piece,
    pub point: Point,
    pub direction: Direction,
}

impl Placement {
    pub fn new(piece: Piece, point: Point, direction: Direction) -> Self {
        Self {
            piece,
            point,
            direction,
        }
    }

    pub fn assignments(&self) -> Vec<Assignment> {
        let a = self.piece.left();
        let b = self.piece.right();
        match self.direction {
            Direction::North => vec![
                Assignment::new(a, Point::new(self.point.x, self.point.y + 1)),
                Assignment::new(b, self.point),
            ],
            Direction::East => vec![
                Assignment::new(a, self.point),
                Assignment::new(b, Point::new(self.point.x + 1, self.point.y)),
            ],
            Direction::South => vec![
                Assignment::new(a, self.point),
                Assignment::new(b, Point::new(self.point.x, self.point.y + 1)),
            ],
            Direction::West => vec![
                Assignment::new(a, Point::new(self.point.x + 1, self.point.y)),
                Assignment::new(b, self.point),
            ],
        }
    }

    pub fn points(&self) -> HashSet<Point> {
        self.assignments()
            .into_iter()
            .map(|assignment| assignment.point)
            .collect()
    }
}

impl fmt::Display for Placement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} @ {} heading {}",
            self.piece, self.point, self.direction
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Direction, Placement};
    use crate::model::{piece::Piece, pips::Pips, point::Point};

    #[test]
    fn points_sets_include_expected_locations() {
        let piece = Piece::new(Pips::new(0).unwrap(), Pips::new(1).unwrap());
        let placement = Placement::new(piece, Point::new(0, 0), Direction::North);
        let pts = placement.points();
        assert!(pts.contains(&Point::new(0, 0)));
        assert!(pts.contains(&Point::new(0, 1)));
    }
}
