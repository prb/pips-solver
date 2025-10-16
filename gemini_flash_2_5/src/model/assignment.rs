use crate::model::pips::Pips;
use crate::model::point::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assignment(pub Pips, pub Point);
