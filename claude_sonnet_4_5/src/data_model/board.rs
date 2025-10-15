// Board - represents a set of points on the grid

use super::placement::Placement;
use super::point::Point;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    points: HashSet<Point>,
}

impl Board {
    pub fn new(points: HashSet<Point>) -> Self {
        Board { points }
    }

    // pub fn empty() -> Self {
    //     Board {
    //         points: HashSet::new(),
    //     }
    // }

    pub fn points(&self) -> &HashSet<Point> {
        &self.points
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Reduces a board by removing points from a placement (reduceB from spec)
    pub fn reduce_b(&self, placement: &Placement) -> Result<Self, String> {
        let placement_points = placement.points();

        // Check if all placement points are in the board
        if !placement_points.is_subset(&self.points) {
            return Err(format!(
                "Placement {:?} has at least one point outside of the board.",
                placement
            ));
        }

        // Remove the placement points from the board
        let new_points: HashSet<Point> = self
            .points
            .difference(&placement_points)
            .copied()
            .collect();

        Ok(Board::new(new_points))
    }
}
