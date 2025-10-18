// Placement - represents a piece placed at a point in a direction

use super::assignment::Assignment;
use super::direction::Direction;
use super::piece::Piece;
use super::point::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Placement {
    pub piece: Piece,
    pub point: Point,
    pub direction: Direction,
}

impl Placement {
    pub fn new(piece: Piece, point: Point, direction: Direction) -> Self {
        Placement {
            piece,
            point,
            direction,
        }
    }

    pub fn assignments(&self) -> [Assignment; 2] {
        let (x, y) = (self.point.x, self.point.y);
        let p1 = self.piece.pips1();
        let p2 = self.piece.pips2();

        match self.direction {
            Direction::North => [
                Assignment::new(p1, Point::new(x, y + 1)),
                Assignment::new(p2, Point::new(x, y)),
            ],
            Direction::East => [
                Assignment::new(p1, Point::new(x, y)),
                Assignment::new(p2, Point::new(x + 1, y)),
            ],
            Direction::South => [
                Assignment::new(p1, Point::new(x, y)),
                Assignment::new(p2, Point::new(x, y + 1)),
            ],
            Direction::West => [
                Assignment::new(p1, Point::new(x + 1, y)),
                Assignment::new(p2, Point::new(x, y)),
            ],
        }
    }

    pub fn points(&self) -> [Point; 2] {
        let assignments = self.assignments();
        [assignments[0].point, assignments[1].point]
    }
}
