// Board - represents a set of points on the grid

use super::placement::Placement;
use super::point::Point;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    points: Arc<HashSet<Point>>,
}

impl Board {
    pub fn new(points: HashSet<Point>) -> Self {
        Board {
            points: Arc::new(points),
        }
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
        if !placement_points.iter().all(|p| self.points.contains(p)) {
            return Err(format!(
                "Placement {:?} has at least one point outside of the board.",
                placement
            ));
        }

        // Use Arc copy-on-write for efficient cloning
        let mut new_points = Arc::clone(&self.points);
        let points_mut = Arc::make_mut(&mut new_points);

        for point in &placement_points {
            points_mut.remove(point);
        }

        Ok(Board { points: new_points })
    }
}
