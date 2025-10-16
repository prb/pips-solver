use super::{assignment::Assignment, pips::Pips, placement::Placement, point::Point};
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constraint {
    AllSame {
        expected: Option<Pips>,
        points: HashSet<Point>,
    },
    AllDifferent {
        excluded: HashSet<Pips>,
        points: HashSet<Point>,
    },
    Exactly {
        target: u32,
        points: HashSet<Point>,
    },
    LessThan {
        target: u32,
        points: HashSet<Point>,
    },
    MoreThan {
        target: u32,
        points: HashSet<Point>,
    },
}

pub type ConstraintSet = Vec<Constraint>;

impl Constraint {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Constraint::AllSame { points, .. } => {
                if points.is_empty() {
                    Err("AllSame constraint must reference at least one point.".to_string())
                } else {
                    Ok(())
                }
            }
            Constraint::AllDifferent { excluded, points } => {
                if points.is_empty() {
                    return Err(
                        "AllDifferent constraint must reference at least one point.".to_string()
                    );
                }
                if points.len() == 1 && excluded.is_empty() {
                    return Err(
                        "AllDifferent constraint with one point must exclude at least one pip."
                            .to_string(),
                    );
                }
                if excluded.len() + points.len() > (Pips::MAX as usize + 1) {
                    return Err("AllDifferent constraint excludes too many pips.".to_string());
                }
                Ok(())
            }
            Constraint::Exactly { target, points } => {
                Self::validate_numeric(*target, points, true, "Exactly")
            }
            Constraint::LessThan { target, points } => {
                if *target == 0 {
                    return Err("LessThan target must be positive.".to_string());
                }
                if *target >= (points.len() as u32) * (Pips::MAX as u32) {
                    return Err(
                        "LessThan target must be less than maximum achievable sum.".to_string()
                    );
                }
                Ok(())
            }
            Constraint::MoreThan { target, points } => {
                Self::validate_numeric(*target, points, true, "MoreThan")
            }
        }
    }

    fn validate_numeric(
        target: u32,
        points: &HashSet<Point>,
        allow_zero: bool,
        label: &str,
    ) -> Result<(), String> {
        if points.is_empty() {
            return Err(format!(
                "{} constraint must reference at least one point.",
                label
            ));
        }
        if !allow_zero && target == 0 {
            return Err(format!("{} target must be positive.", label));
        }
        let max = (points.len() as u32) * (Pips::MAX as u32);
        if target > max {
            return Err(format!(
                "{} target exceeds achievable sum for the given points.",
                label
            ));
        }
        Ok(())
    }

    pub fn points(&self) -> &HashSet<Point> {
        match self {
            Constraint::AllSame { points, .. }
            | Constraint::AllDifferent { points, .. }
            | Constraint::Exactly { points, .. }
            | Constraint::LessThan { points, .. }
            | Constraint::MoreThan { points, .. } => points,
        }
    }

    pub fn reduce_assignment(&self, assignment: &Assignment) -> Result<Option<Constraint>, String> {
        if !self.points().contains(&assignment.point) {
            return Ok(Some(self.clone()));
        }
        match self {
            Constraint::AllDifferent { excluded, points } => {
                if excluded.contains(&assignment.pips) {
                    return Err(format!("The pip {} is already used.", assignment.pips));
                }
                let mut remaining_points = points.clone();
                remaining_points.remove(&assignment.point);
                let mut new_excluded = excluded.clone();
                new_excluded.insert(assignment.pips);
                if remaining_points.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(Constraint::AllDifferent {
                        excluded: new_excluded,
                        points: remaining_points,
                    }))
                }
            }
            Constraint::AllSame { expected, points } => {
                let size = points.len();
                let mut remaining = points.clone();
                remaining.remove(&assignment.point);
                match expected {
                    Some(target) => {
                        if assignment.pips != *target {
                            Err(format!(
                                "The pip {} is not the same as the expected pip {}.",
                                assignment.pips, target
                            ))
                        } else if size == 1 {
                            Ok(None)
                        } else if size == 2 {
                            Ok(Some(Constraint::Exactly {
                                target: target.value() as u32,
                                points: remaining,
                            }))
                        } else {
                            Ok(Some(Constraint::AllSame {
                                expected: Some(*target),
                                points: remaining,
                            }))
                        }
                    }
                    None => {
                        if size == 1 {
                            Ok(None)
                        } else {
                            let pip = assignment.pips;
                            if size == 2 {
                                Ok(Some(Constraint::Exactly {
                                    target: pip.value() as u32,
                                    points: remaining,
                                }))
                            } else {
                                Ok(Some(Constraint::AllSame {
                                    expected: Some(pip),
                                    points: remaining,
                                }))
                            }
                        }
                    }
                }
            }
            Constraint::Exactly { target, points } => {
                let mut remaining = points.clone();
                remaining.remove(&assignment.point);
                let size = points.len();
                let pip_value = assignment.pips.value() as u32;
                if size == 1 {
                    if pip_value == *target {
                        Ok(None)
                    } else {
                        Err(format!(
                            "The pips {} is not the same as the expected pip {}.",
                            assignment.pips, target
                        ))
                    }
                } else if pip_value > *target {
                    Err(format!(
                        "The pip {} exceeds the expected exact sum {}.",
                        assignment.pips, target
                    ))
                } else {
                    let remaining_target = target - pip_value;
                    let max_possible = remaining.len() as u32 * (Pips::MAX as u32);
                    if remaining_target > max_possible {
                        Err(format!(
                            "The remaining sum {} is unachievable with {} points.",
                            remaining_target,
                            remaining.len()
                        ))
                    } else {
                        Ok(Some(Constraint::Exactly {
                            target: remaining_target,
                            points: remaining,
                        }))
                    }
                }
            }
            Constraint::LessThan { target, points } => {
                let mut remaining = points.clone();
                remaining.remove(&assignment.point);
                let size = points.len();
                let pip_value = assignment.pips.value() as u32;
                if pip_value >= *target {
                    return Err(format!(
                        "The pips {} is not less than the target sum {}.",
                        assignment.pips, target
                    ));
                }
                if size == 1 {
                    Ok(None)
                } else {
                    let remaining_target = target - pip_value;
                    if size == 2 && remaining_target == 1 {
                        Ok(Some(Constraint::Exactly {
                            target: 0,
                            points: remaining,
                        }))
                    } else {
                        Ok(Some(Constraint::LessThan {
                            target: remaining_target,
                            points: remaining,
                        }))
                    }
                }
            }
            Constraint::MoreThan { target, points } => {
                let mut remaining = points.clone();
                remaining.remove(&assignment.point);
                let size = points.len();
                let pip_value = assignment.pips.value() as i32;
                let remaining_points = remaining.len();
                if size == 1 {
                    if pip_value > *target as i32 {
                        Ok(None)
                    } else {
                        Err(format!(
                            "The pips {} is less than the minimum required sum of {}.",
                            assignment.pips,
                            target + 1
                        ))
                    }
                } else {
                    let remaining_target = *target as i32 - pip_value;
                    if remaining_points == 1 && remaining_target == 5 {
                        Ok(Some(Constraint::Exactly {
                            target: 6,
                            points: remaining,
                        }))
                    } else if remaining_target < 0 {
                        Ok(None)
                    } else {
                        let max_possible = (remaining_points as i32) * (Pips::MAX as i32);
                        if remaining_target >= max_possible {
                            Err(format!(
                                "The remaining sum {} is unachievable with {} points.",
                                remaining_target, remaining_points
                            ))
                        } else {
                            Ok(Some(Constraint::MoreThan {
                                target: remaining_target as u32,
                                points: remaining,
                            }))
                        }
                    }
                }
            }
        }
    }

    pub fn reduce_placement(&self, placement: &Placement) -> Result<Option<Constraint>, String> {
        placement
            .assignments()
            .iter()
            .try_fold(Some(self.clone()), |current, assignment| match current {
                None => Ok(None),
                Some(constraint) => constraint.reduce_assignment(assignment),
            })
    }
}

pub fn reduce_constraints(
    constraints: &[Constraint],
    placement: &Placement,
) -> Result<Vec<Constraint>, String> {
    let mut reduced = Vec::new();
    for constraint in constraints {
        match constraint.reduce_placement(placement) {
            Ok(Some(next)) => reduced.push(next),
            Ok(None) => {}
            Err(_) => {
                return Err("At least one constraint was violated by the placement.".to_string());
            }
        }
    }
    Ok(reduced)
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::AllSame { expected, points } => match expected {
                Some(pips) => write!(f, "AllSame(Some({}), {:?})", pips, points),
                None => write!(f, "AllSame(None, {:?})", points),
            },
            Constraint::AllDifferent { excluded, points } => {
                write!(f, "AllDifferent({:?}, {:?})", excluded, points)
            }
            Constraint::Exactly { target, points } => {
                write!(f, "Exactly({}, {:?})", target, points)
            }
            Constraint::LessThan { target, points } => {
                write!(f, "LessThan({}, {:?})", target, points)
            }
            Constraint::MoreThan { target, points } => {
                write!(f, "MoreThan({}, {:?})", target, points)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Constraint, reduce_constraints};
    use crate::model::{
        direction::Direction, piece::Piece, pips::Pips, placement::Placement, point::Point,
    };
    use std::collections::HashSet;

    fn piece(a: u8, b: u8) -> Piece {
        Piece::new(Pips::new(a).unwrap(), Pips::new(b).unwrap())
    }

    fn set_of(points: &[Point]) -> HashSet<Point> {
        points.iter().copied().collect()
    }

    #[test]
    fn all_same_mismatch_fails() {
        let constraint = Constraint::AllSame {
            expected: Some(Pips::new(3).unwrap()),
            points: set_of(&[Point::new(0, 0)]),
        };
        let placement = Placement::new(piece(3, 4), Point::new(0, 0), Direction::North);
        let result = reduce_constraints(&[constraint], &placement);
        assert!(result.is_err());
    }

    #[test]
    fn all_different_consumes_points() {
        let constraint = Constraint::AllDifferent {
            excluded: HashSet::new(),
            points: set_of(&[Point::new(0, 0), Point::new(1, 0)]),
        };
        let placement = Placement::new(piece(1, 2), Point::new(0, 0), Direction::East);
        let reduced = reduce_constraints(&[constraint], &placement).unwrap();
        assert!(reduced.is_empty());
    }
}
