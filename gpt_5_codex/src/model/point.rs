use std::fmt;

/// Represents a coordinate on the board grid.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::Point;

    #[test]
    fn point_is_constructed() {
        let p = Point::new(1, 2);
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
    }
}
