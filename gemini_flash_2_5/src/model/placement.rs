use crate::model::assignment::Assignment;
use crate::model::direction::Direction;
use crate::model::piece::Piece;
use crate::model::point::Point;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub piece: Piece,
    pub point: Point,
    pub direction: Direction,
}

impl fmt::Display for Placement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {:?}", self.piece, self.point, self.direction)
    }
}

impl Placement {
    pub fn assignments(&self) -> [Assignment; 2] {
        let p1 = self.piece.p1();
        let p2 = self.piece.p2();
        let Point(x, y) = self.point;

        match self.direction {
            Direction::North => [Assignment(p1, Point(x, y + 1)), Assignment(p2, Point(x, y))],
            Direction::East => [Assignment(p1, Point(x, y)), Assignment(p2, Point(x + 1, y))],
            Direction::South => [Assignment(p1, Point(x, y)), Assignment(p2, Point(x, y + 1))],
            Direction::West => [Assignment(p1, Point(x + 1, y)), Assignment(p2, Point(x, y))],
        }
    }

    pub fn points(&self) -> HashSet<Point> {
        self.assignments().iter().map(|a| a.1).collect()
    }
}
