use super::point::Point;
use once_cell::sync::Lazy;
use std::collections::HashSet;

/// Represents the playable board as a set of points.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    points: HashSet<Point>,
}

impl Board {
    pub fn new(points: HashSet<Point>) -> Self {
        Self { points }
    }

    pub fn points(&self) -> &HashSet<Point> {
        &self.points
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn contains_all(&self, other: &HashSet<Point>) -> bool {
        other.iter().all(|point| self.points.contains(point))
    }

    pub fn remove_points(&self, to_remove: &HashSet<Point>) -> Result<Board, String> {
        if !self.contains_all(to_remove) {
            return Err("Placement has at least one point outside of the board.".to_string());
        }
        let mut next = self.points.clone();
        for point in to_remove {
            next.remove(point);
        }
        Ok(Board::new(next))
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new(HashSet::new())
    }
}

#[allow(dead_code)]
pub static EMPTY_BOARD: Lazy<Board> = Lazy::new(Board::default);

#[cfg(test)]
mod tests {
    use super::{Board, Point};
    use std::collections::HashSet;

    #[test]
    fn remove_points_succeeds_for_subset() {
        let mut pts = HashSet::new();
        pts.insert(Point::new(0, 0));
        pts.insert(Point::new(0, 1));
        let board = Board::new(pts.clone());

        let mut take = HashSet::new();
        take.insert(Point::new(0, 1));
        let next = board.remove_points(&take).unwrap();
        assert_eq!(next.len(), 1);
        assert!(next.points().contains(&Point::new(0, 0)));
    }

    #[test]
    fn remove_points_errors_for_non_subset() {
        let board = Board::default();
        let mut take = HashSet::new();
        take.insert(Point::new(0, 0));
        assert!(board.remove_points(&take).is_err());
    }
}
