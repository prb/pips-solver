use crate::model::point::Point;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board(pub HashSet<Point>);

impl Board {
    pub fn new(points: HashSet<Point>) -> Self {
        Board(points)
    }

    pub fn points(&self) -> &HashSet<Point> {
        &self.0
    }
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref EMPTY_BOARD: Board = Board(HashSet::new());
}
