// Assignment - represents a pips assigned to a point

use super::pips::Pips;
use super::point::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Assignment {
    pub pips: Pips,
    pub point: Point,
}

impl Assignment {
    pub fn new(pips: Pips, point: Point) -> Self {
        Assignment { pips, point }
    }
}
