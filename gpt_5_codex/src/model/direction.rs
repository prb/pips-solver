use std::fmt;

/// Compass direction used when placing a domino.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    #[allow(dead_code)]
    pub const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Direction::North => "North",
            Direction::East => "East",
            Direction::South => "South",
            Direction::West => "West",
        };
        write!(f, "{}", label)
    }
}

#[cfg(test)]
mod tests {
    use super::Direction;

    #[test]
    fn all_directions_listed() {
        assert_eq!(Direction::ALL.len(), 4);
    }
}
