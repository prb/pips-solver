// Constraint - represents game constraints

use super::assignment::Assignment;
use super::placement::Placement;
use super::pips::Pips;
use super::point::Point;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    Empty,
    AllSame {
        target: Option<Pips>,
        points: HashSet<Point>,
    },
    AllDifferent {
        excluded: HashSet<Pips>,
        points: HashSet<Point>,
    },
    LessThan {
        target: usize,
        points: HashSet<Point>,
    },
    Exactly {
        target: usize,
        points: HashSet<Point>,
    },
    MoreThan {
        target: usize,
        points: HashSet<Point>,
    },
}

impl Constraint {
    /// Creates an AllSame constraint with invariant checking
    pub fn all_same(target: Option<Pips>, points: HashSet<Point>) -> Result<Self, String> {
        // AllSame invariant: set of points must have size at least 2
        if points.len() < 2 {
            return Err(format!(
                "AllSame constraint requires at least 2 points, got {}",
                points.len()
            ));
        }

        Ok(Constraint::AllSame { target, points })
    }

    /// Creates an AllDifferent constraint with invariant checking
    pub fn all_different(excluded: HashSet<Pips>, points: HashSet<Point>) -> Result<Self, String> {
        // AllDifferent invariant 1: size of excluded + size of points <= 7 (range of possible pips)
        if excluded.len() + points.len() > 7 {
            return Err(format!(
                "AllDifferent constraint invalid: excluded.len() ({}) + points.len() ({}) > 7",
                excluded.len(),
                points.len()
            ));
        }

        // AllDifferent invariant 2: if excluded is empty, points must have size at least 2
        if excluded.is_empty() && points.len() < 2 {
            return Err(format!(
                "AllDifferent constraint with empty excluded set requires at least 2 points, got {}",
                points.len()
            ));
        }

        // AllDifferent invariant 3: points must be non-empty
        if points.is_empty() {
            return Err("AllDifferent constraint requires at least 1 point".to_string());
        }

        Ok(Constraint::AllDifferent { excluded, points })
    }

    /// Creates a LessThan constraint with invariant checking
    pub fn less_than(target: usize, points: HashSet<Point>) -> Result<Self, String> {
        // LessThan invariant 1: target must be greater than 0
        if target == 0 {
            return Err("LessThan constraint requires target > 0".to_string());
        }

        // LessThan invariant 2: target must be strictly less than 6 * points.len()
        if target >= 6 * points.len() {
            return Err(format!(
                "LessThan constraint invalid: target ({}) >= 6 * points.len() ({})",
                target,
                6 * points.len()
            ));
        }

        if points.is_empty() {
            return Err("LessThan constraint requires at least 1 point".to_string());
        }

        Ok(Constraint::LessThan { target, points })
    }

    /// Creates an Exactly constraint with invariant checking
    pub fn exactly(target: usize, points: HashSet<Point>) -> Result<Self, String> {
        // Exactly invariant: target must not be larger than 6 * points.len()
        if target > 6 * points.len() {
            return Err(format!(
                "Exactly constraint invalid: target ({}) > 6 * points.len() ({})",
                target,
                6 * points.len()
            ));
        }

        if points.is_empty() {
            return Err("Exactly constraint requires at least 1 point".to_string());
        }

        Ok(Constraint::Exactly { target, points })
    }

    /// Creates a MoreThan constraint with invariant checking
    pub fn more_than(target: usize, points: HashSet<Point>) -> Result<Self, String> {
        // MoreThan invariant: target must not be larger than 6 * points.len()
        if target > 6 * points.len() {
            return Err(format!(
                "MoreThan constraint invalid: target ({}) > 6 * points.len() ({})",
                target,
                6 * points.len()
            ));
        }

        if points.is_empty() {
            return Err("MoreThan constraint requires at least 1 point".to_string());
        }

        Ok(Constraint::MoreThan { target, points })
    }

    /// Reduces a constraint by applying an assignment (reduceA from spec)
    pub fn reduce_a(&self, assignment: &Assignment) -> Result<Self, String> {
        match self {
            // Empty constraint reduces to itself
            Constraint::Empty => Ok(Constraint::Empty),

            // No change if the point is outside the constraint
            Constraint::AllSame { target: _, points } if !points.contains(&assignment.point) => {
                Ok(self.clone())
            }
            Constraint::AllDifferent { excluded: _, points } if !points.contains(&assignment.point) => {
                Ok(self.clone())
            }
            Constraint::LessThan { target: _, points } if !points.contains(&assignment.point) => {
                Ok(self.clone())
            }
            Constraint::Exactly { target: _, points } if !points.contains(&assignment.point) => {
                Ok(self.clone())
            }
            Constraint::MoreThan { target: _, points } if !points.contains(&assignment.point) => {
                Ok(self.clone())
            }

            // AllDifferent
            Constraint::AllDifferent { excluded, points } => {
                if excluded.contains(&assignment.pips) {
                    return Err(format!("The pip {} is already used.", assignment.pips.value()));
                }

                let mut new_excluded = excluded.clone();
                new_excluded.insert(assignment.pips);
                let mut new_points = points.clone();
                new_points.remove(&assignment.point);

                if new_points.len() == 0 {
                    Ok(Constraint::Empty)
                } else {
                    Ok(Constraint::AllDifferent {
                        excluded: new_excluded,
                        points: new_points,
                    })
                }
            }

            // AllSame
            Constraint::AllSame { target, points } => {
                if let Some(target_val) = target {
                    if assignment.pips != *target_val {
                        return Err(format!(
                            "The pip {} is not the same as the expected pip {}.",
                            assignment.pips.value(),
                            target_val.value()
                        ));
                    }
                }

                let mut new_points = points.clone();
                new_points.remove(&assignment.point);

                match new_points.len() {
                    0 => Ok(Constraint::Empty),
                    1 => {
                        // Transition to Exactly constraint
                        let target_val = target.unwrap_or(assignment.pips);
                        Ok(Constraint::Exactly {
                            target: target_val.value() as usize,
                            points: new_points,
                        })
                    }
                    _ => Ok(Constraint::AllSame {
                        target: Some(target.unwrap_or(assignment.pips)),
                        points: new_points,
                    }),
                }
            }

            // Exactly
            Constraint::Exactly { target, points } => {
                if points.len() == 1 {
                    if assignment.pips.value() as usize != *target {
                        return Err(format!(
                            "The pips {} is not the same as the expected pip {}.",
                            assignment.pips.value(),
                            target
                        ));
                    }
                    return Ok(Constraint::Empty);
                }

                if assignment.pips.value() as usize > *target {
                    return Err(format!(
                        "The pip {} exceeds the expected exact sum {}.",
                        assignment.pips.value(),
                        target
                    ));
                }

                let mut new_points = points.clone();
                new_points.remove(&assignment.point);
                let new_target = target - assignment.pips.value() as usize;

                // Check if the new target is achievable with the remaining points
                let max_possible = new_points.len() * 6;
                if new_target > max_possible {
                    return Err(format!(
                        "The remaining sum {} is unachievable with {} points.",
                        new_target,
                        new_points.len()
                    ));
                }

                Ok(Constraint::Exactly {
                    target: new_target,
                    points: new_points,
                })
            }

            // LessThan
            Constraint::LessThan { target, points } => {
                if assignment.pips.value() as usize >= *target {
                    return Err(format!(
                        "The pips {} is not less than the target sum {}.",
                        assignment.pips.value(),
                        target
                    ));
                }

                let mut new_points = points.clone();
                new_points.remove(&assignment.point);

                if new_points.len() == 0 {
                    Ok(Constraint::Empty)
                } else if *target - assignment.pips.value() as usize == 1 && new_points.len() == 1 {
                    // Special case: remaining sum must be exactly 0
                    Ok(Constraint::Exactly {
                        target: 0,
                        points: new_points,
                    })
                } else {
                    Ok(Constraint::LessThan {
                        target: target - assignment.pips.value() as usize,
                        points: new_points,
                    })
                }
            }

            // MoreThan
            Constraint::MoreThan { target, points } => {
                // Single point case: must exceed target
                if points.len() == 1 {
                    if assignment.pips.value() as usize <= *target {
                        return Err(format!(
                            "The pips {} is less than the minimum required sum of {}.",
                            assignment.pips.value(),
                            target + 1
                        ));
                    }
                    return Ok(Constraint::Empty);
                }

                let mut new_points = points.clone();
                new_points.remove(&assignment.point);
                let new_target = target.saturating_sub(assignment.pips.value() as usize);

                // Special case: if new_target == 5 and we have 1 point left, it must be exactly 6
                if new_target == 5 && new_points.len() == 1 {
                    Ok(Constraint::Exactly {
                        target: 6,
                        points: new_points,
                    })
                } else {
                    // General case: reduce the target by the assignment value
                    Ok(Constraint::MoreThan {
                        target: new_target,
                        points: new_points,
                    })
                }
            }
        }
    }

    /// Reduces a constraint by applying all assignments from a placement (reduceP from spec)
    pub fn reduce_p(&self, placement: &Placement) -> Result<Self, String> {
        let assignments = placement.assignments();
        let mut current = self.clone();

        for assignment in assignments {
            current = current.reduce_a(&assignment)?;
        }

        Ok(current)
    }
}

/// Reduces a collection of constraints by applying a placement (reduceCs from spec)
pub fn reduce_cs(constraints: &[Constraint], placement: &Placement) -> Result<Vec<Constraint>, String> {
    let mut result = Vec::new();

    for constraint in constraints {
        match constraint.reduce_p(placement)? {
            Constraint::Empty => {
                // Filter out empty constraints
            }
            reduced => result.push(reduced),
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::piece::Piece;

    fn make_points(coords: &[(usize, usize)]) -> HashSet<Point> {
        coords.iter().map(|&(x, y)| Point::new(x, y)).collect()
    }

    #[test]
    fn test_all_same_valid() {
        let points = make_points(&[(0, 0), (1, 0)]);
        assert!(Constraint::all_same(None, points).is_ok());

        let points = make_points(&[(0, 0), (1, 0), (2, 0)]);
        let pips = Pips::new(3).unwrap();
        assert!(Constraint::all_same(Some(pips), points).is_ok());
    }

    #[test]
    fn test_all_same_invalid_single_point() {
        let points = make_points(&[(0, 0)]);
        assert!(Constraint::all_same(None, points).is_err());
    }

    #[test]
    fn test_all_different_valid() {
        let points = make_points(&[(0, 0), (1, 0)]);
        let excluded = HashSet::new();
        assert!(Constraint::all_different(excluded, points).is_ok());
    }

    #[test]
    fn test_all_different_invalid_too_many() {
        let points = make_points(&[(0, 0), (1, 0), (2, 0)]);
        let mut excluded = HashSet::new();
        for i in 0..=4 {
            excluded.insert(Pips::new(i).unwrap());
        }
        // 5 excluded + 3 points = 8 > 7
        assert!(Constraint::all_different(excluded, points).is_err());
    }

    #[test]
    fn test_all_different_invalid_single_point_no_excluded() {
        let points = make_points(&[(0, 0)]);
        let excluded = HashSet::new();
        assert!(Constraint::all_different(excluded, points).is_err());
    }

    #[test]
    fn test_less_than_valid() {
        let points = make_points(&[(0, 0), (1, 0)]);
        assert!(Constraint::less_than(5, points).is_ok());
    }

    #[test]
    fn test_less_than_invalid_zero() {
        let points = make_points(&[(0, 0)]);
        assert!(Constraint::less_than(0, points).is_err());
    }

    #[test]
    fn test_less_than_invalid_too_large() {
        let points = make_points(&[(0, 0), (1, 0), (2, 0)]);
        // target must be < 6 * 3 = 18
        assert!(Constraint::less_than(18, points).is_err());
    }

    #[test]
    fn test_exactly_valid() {
        let points = make_points(&[(0, 0), (1, 0), (2, 0)]);
        assert!(Constraint::exactly(0, points.clone()).is_ok());
        assert!(Constraint::exactly(18, points).is_ok());
    }

    #[test]
    fn test_exactly_invalid_too_large() {
        let points = make_points(&[(0, 0), (1, 0), (2, 0)]);
        // target must be <= 6 * 3 = 18
        assert!(Constraint::exactly(19, points).is_err());
    }

    #[test]
    fn test_more_than_valid() {
        let points = make_points(&[(0, 0), (1, 0), (2, 0)]);
        assert!(Constraint::more_than(5, points).is_ok());
    }

    #[test]
    fn test_more_than_invalid_too_large() {
        let points = make_points(&[(0, 0), (1, 0), (2, 0)]);
        // target must be <= 6 * 3 = 18
        assert!(Constraint::more_than(19, points).is_err());
    }

    #[test]
    fn test_reduce_a_empty() {
        let c = Constraint::Empty;
        let a = Assignment::new(Pips::new(1).unwrap(), Point::new(0, 0));
        assert_eq!(c.reduce_a(&a).unwrap(), Constraint::Empty);
    }

    #[test]
    fn test_reduce_a_point_outside() {
        let points = make_points(&[(0, 0), (1, 0)]);
        let c = Constraint::exactly(5, points).unwrap();
        let a = Assignment::new(Pips::new(1).unwrap(), Point::new(2, 2));
        assert_eq!(c.reduce_a(&a).unwrap(), c);
    }

    #[test]
    fn test_reduce_a_all_different() {
        let points = make_points(&[(0, 0), (1, 0), (2, 0)]);
        let c = Constraint::all_different(HashSet::new(), points).unwrap();
        let a = Assignment::new(Pips::new(1).unwrap(), Point::new(0, 0));

        let result = c.reduce_a(&a).unwrap();
        match &result {
            Constraint::AllDifferent { excluded, points } => {
                assert!(excluded.contains(&Pips::new(1).unwrap()));
                assert_eq!(points.len(), 2);
            }
            _ => panic!("Expected AllDifferent"),
        }

        // Test violation: using already excluded pip
        let result2 = result.reduce_a(&Assignment::new(Pips::new(1).unwrap(), Point::new(1, 0)));
        assert!(result2.is_err());
    }

    #[test]
    fn test_reduce_a_all_same() {
        // Specification example from strategy.md lines 187-200
        let points2 = make_points(&[(0, 0), (0, 1)]);
        let c2 = Constraint::all_same(None, points2).unwrap();
        let a2 = Assignment::new(Pips::new(0).unwrap(), Point::new(0, 1));

        let result = c2.reduce_a(&a2).unwrap();
        // After first assignment, should become Exactly(0, {(0,0)})
        match &result {
            Constraint::Exactly { target, points } => {
                assert_eq!(*target, 0);
                assert_eq!(points.len(), 1);
            }
            _ => panic!("Expected Exactly, got {:?}", result),
        }

        // Now assigning 1 to (0,0) should fail
        let a3 = Assignment::new(Pips::new(1).unwrap(), Point::new(0, 0));
        assert!(result.reduce_a(&a3).is_err());
    }

    #[test]
    fn test_reduce_a_exactly() {
        let points = make_points(&[(0, 0), (1, 0)]);
        let c = Constraint::exactly(5, points).unwrap();
        let a = Assignment::new(Pips::new(2).unwrap(), Point::new(0, 0));

        let result = c.reduce_a(&a).unwrap();
        match result {
            Constraint::Exactly { target, points } => {
                assert_eq!(target, 3);
                assert_eq!(points.len(), 1);
            }
            _ => panic!("Expected Exactly"),
        }

        // Test violation: exceeding target
        let points2 = make_points(&[(0, 0), (1, 0)]);
        let c2 = Constraint::exactly(5, points2).unwrap();
        let a2 = Assignment::new(Pips::new(6).unwrap(), Point::new(0, 0));
        assert!(c2.reduce_a(&a2).is_err());

        // Test optimization: unachievable remaining sum
        let points3 = make_points(&[(0, 0), (1, 0)]);
        let c3 = Constraint::exactly(10, points3).unwrap();
        let a3 = Assignment::new(Pips::new(3).unwrap(), Point::new(0, 0));
        // Remaining sum would be 7, but only 1 point left (max 6)
        assert!(c3.reduce_a(&a3).is_err());
    }

    #[test]
    fn test_reduce_a_less_than() {
        let points = make_points(&[(0, 0), (1, 0)]);
        let c = Constraint::less_than(5, points).unwrap();
        let a = Assignment::new(Pips::new(2).unwrap(), Point::new(0, 0));

        let result = c.reduce_a(&a).unwrap();
        match result {
            Constraint::LessThan { target, points } => {
                assert_eq!(target, 3);
                assert_eq!(points.len(), 1);
            }
            _ => panic!("Expected LessThan"),
        }

        // Test special case: target - pips == 1 with one point left -> Exactly(0)
        let points2 = make_points(&[(0, 0), (1, 0)]);
        let c2 = Constraint::less_than(3, points2).unwrap();
        let a2 = Assignment::new(Pips::new(2).unwrap(), Point::new(0, 0));

        let result2 = c2.reduce_a(&a2).unwrap();
        match result2 {
            Constraint::Exactly { target, points } => {
                assert_eq!(target, 0);
                assert_eq!(points.len(), 1);
            }
            _ => panic!("Expected Exactly(0), got {:?}", result2),
        }
    }

    #[test]
    fn test_reduce_a_more_than() {
        let points = make_points(&[(0, 0), (1, 0)]);
        let c = Constraint::more_than(3, points).unwrap();
        let a = Assignment::new(Pips::new(2).unwrap(), Point::new(0, 0));

        let result = c.reduce_a(&a).unwrap();
        match result {
            Constraint::MoreThan { target, points } => {
                assert_eq!(target, 1);
                assert_eq!(points.len(), 1);
            }
            _ => panic!("Expected MoreThan"),
        }

        // Test violation: single point with pips <= target
        let points2 = make_points(&[(0, 0)]);
        let c2 = Constraint::more_than(5, points2).unwrap();
        let a2 = Assignment::new(Pips::new(3).unwrap(), Point::new(0, 0));
        assert!(c2.reduce_a(&a2).is_err());

        // Test special case: (v1-a.pips) == 5 and size(P) == 2 -> Exactly(6)
        let points3 = make_points(&[(0, 0), (1, 0)]);
        let c3 = Constraint::more_than(6, points3).unwrap();
        let a3 = Assignment::new(Pips::new(1).unwrap(), Point::new(0, 0));

        let result3 = c3.reduce_a(&a3).unwrap();
        match result3 {
            Constraint::Exactly { target, points } => {
                assert_eq!(target, 6);
                assert_eq!(points.len(), 1);
            }
            _ => panic!("Expected Exactly(6), got {:?}", result3),
        }
    }

    #[test]
    fn test_reduce_p_example_from_spec() {
        // Example from specification (strategy.md lines 187-200)
        use super::super::direction::Direction;

        let points = make_points(&[(0, 0), (0, 1)]);
        let constraint = Constraint::all_same(None, points).unwrap();

        let piece = Piece::new(Pips::new(0).unwrap(), Pips::new(1).unwrap());
        let placement = Placement::new(piece, Point::new(0, 0), Direction::North);

        // This should fail because:
        // - First assignment: (0, (0,1)) sets target to Some(0)
        // - Then: AllSame(Some(0), {(0,0)}) -> Exactly(0, {(0,0)})
        // - Second assignment: (1, (0,0)) violates Exactly(0, {(0,0)})
        let result = constraint.reduce_p(&placement);
        assert!(result.is_err());
    }

    #[test]
    fn test_reduce_cs() {
        use super::super::direction::Direction;

        let points1 = make_points(&[(0, 0)]);
        let c1 = Constraint::exactly(2, points1).unwrap();

        let points2 = make_points(&[(0, 1)]);
        let c2 = Constraint::exactly(3, points2).unwrap();

        let constraints = vec![c1, c2];

        let piece = Piece::new(Pips::new(2).unwrap(), Pips::new(3).unwrap());
        let placement = Placement::new(piece, Point::new(0, 0), Direction::South);

        let result = reduce_cs(&constraints, &placement).unwrap();

        // Both constraints should be satisfied and removed (become Empty)
        assert_eq!(result.len(), 0);
    }
}
