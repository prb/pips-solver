use crate::model::assignment::Assignment;
use crate::model::pips::Pips;
use crate::model::point::Point;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    AllSame(Option<Pips>, HashSet<Point>),
    AllDifferent(HashSet<Pips>, HashSet<Point>),
    LessThan(usize, HashSet<Point>),
    Exactly(usize, HashSet<Point>),
    MoreThan(usize, HashSet<Point>),
}

pub const EMPTY_CONSTRAINT: Option<Constraint> = None;

impl Constraint {
    pub fn new_all_same(pips: Option<Pips>, points: HashSet<Point>) -> Result<Self, String> {
        if points.len() < 2 {
            return Err("AllSame constraint must have at least 2 points".to_string());
        }
        Ok(Constraint::AllSame(pips, points))
    }

    pub fn new_all_different(pips: HashSet<Pips>, points: HashSet<Point>) -> Result<Self, String> {
        if pips.len() + points.len() > 7 {
            return Err(
                "AllDifferent constraint has too many excluded pips for the number of points"
                    .to_string(),
            );
        }
        if pips.is_empty() && points.len() < 2 {
            return Err(
                "AllDifferent constraint with no excluded pips must have at least 2 points"
                    .to_string(),
            );
        }
        Ok(Constraint::AllDifferent(pips, points))
    }

    pub fn new_less_than(value: usize, points: HashSet<Point>) -> Result<Self, String> {
        if value == 0 {
            return Err("LessThan constraint target must be greater than 0".to_string());
        }
        Ok(Constraint::LessThan(value, points))
    }

    pub fn new_exactly(value: usize, points: HashSet<Point>) -> Result<Self, String> {
        if value > 6 * points.len() {
            return Err("Exactly constraint target is unachievable".to_string());
        }
        Ok(Constraint::Exactly(value, points))
    }

    pub fn new_more_than(value: usize, points: HashSet<Point>) -> Result<Self, String> {
        if value >= 6 * points.len() {
            return Err("MoreThan constraint target is unachievable".to_string());
        }
        Ok(Constraint::MoreThan(value, points))
    }

    pub fn reduce_a(self, assignment: &Assignment) -> Result<Option<Constraint>, String> {
        let Assignment(pips, point) = *assignment;

        match self {
            Constraint::AllSame(p, mut points) => {
                if !points.contains(&point) {
                    return Ok(Some(Constraint::AllSame(p, points)));
                }
                points.remove(&point);
                match p {
                    Some(pips_val) => {
                        if pips != pips_val {
                            return Err(format!(
                                "The pip {} is not the same as the expected pip {}",
                                pips, pips_val
                            ));
                        }
                        if points.len() > 1 {
                            Ok(Some(Constraint::AllSame(Some(pips_val), points)))
                        } else if points.len() == 1 {
                            Ok(Some(Constraint::Exactly(pips_val.value() as usize, points)))
                        } else {
                            Ok(None)
                        }
                    }
                    None => {
                        if points.len() > 1 {
                            Ok(Some(Constraint::AllSame(Some(pips), points)))
                        } else if points.len() == 1 {
                            Ok(Some(Constraint::Exactly(pips.value() as usize, points)))
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
            Constraint::AllDifferent(mut p, mut points) => {
                if !points.contains(&point) {
                    return Ok(Some(Constraint::AllDifferent(p, points)));
                }
                if p.contains(&pips) {
                    return Err(format!("The pip {} is already used", pips));
                }
                points.remove(&point);
                p.insert(pips);
                if points.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(Constraint::AllDifferent(p, points)))
                }
            }
            Constraint::Exactly(v, mut points) => {
                if !points.contains(&point) {
                    return Ok(Some(Constraint::Exactly(v, points)));
                }
                points.remove(&point);
                let pips_val = pips.value() as usize;
                if points.is_empty() {
                    if pips_val == v {
                        Ok(None)
                    } else {
                        Err(format!(
                            "The pips {} is not the same as the expected pip {}",
                            pips, v
                        ))
                    }
                } else {
                    if pips_val > v {
                        return Err(format!(
                            "The pip {} exceeds the expected exact sum {}",
                            pips, v
                        ));
                    }
                    let remaining = v - pips_val;
                    if remaining > 6 * points.len() {
                        return Err(format!(
                            "The remaining sum {} is unachievable with {} points",
                            remaining,
                            points.len()
                        ));
                    }
                    Ok(Some(Constraint::Exactly(remaining, points)))
                }
            }
            Constraint::LessThan(v, mut points) => {
                if !points.contains(&point) {
                    return Ok(Some(Constraint::LessThan(v, points)));
                }
                points.remove(&point);
                let pips_val = pips.value() as usize;
                if pips_val >= v {
                    return Err(format!(
                        "The pips {} is not less than the target sum {}",
                        pips, v
                    ));
                }
                if points.is_empty() {
                    Ok(None)
                } else {
                    let remaining = v - pips_val;
                    if remaining == 1 && points.len() == 1 {
                        Ok(Some(Constraint::Exactly(0, points)))
                    } else {
                        Ok(Some(Constraint::LessThan(remaining, points)))
                    }
                }
            }
            Constraint::MoreThan(v, mut points) => {
                if !points.contains(&point) {
                    return Ok(Some(Constraint::MoreThan(v, points)));
                }
                points.remove(&point);
                let pips_val = pips.value() as usize;
                if points.is_empty() {
                    if pips_val > v {
                        Ok(None)
                    } else {
                        Err(format!(
                            "The pips {} is less than the minimum required sum of {}",
                            pips,
                            v + 1
                        ))
                    }
                } else {
                    let remaining = v - pips_val;
                    if remaining == 5 && points.len() == 1 {
                        Ok(Some(Constraint::Exactly(6, points)))
                    } else {
                        Ok(Some(Constraint::MoreThan(remaining, points)))
                    }
                }
            }
        }
    }
}
