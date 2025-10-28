use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pentomino {
    F, I, L, N, P, T, U, V, W, X, Y, Z,
}

impl Pentomino {
    pub fn label(&self) -> char {
        match self {
            Pentomino::F => 'F',
            Pentomino::I => 'I',
            Pentomino::L => 'L',
            Pentomino::N => 'N',
            Pentomino::P => 'P',
            Pentomino::T => 'T',
            Pentomino::U => 'U',
            Pentomino::V => 'V',
            Pentomino::W => 'W',
            Pentomino::X => 'X',
            Pentomino::Y => 'Y',
            Pentomino::Z => 'Z',
        }
    }

    pub fn base_shape(&self) -> Vec<(i32, i32)> {
        match self {
            Pentomino::F => vec![(0, 1), (1, 0), (1, 1), (1, 2), (2, 2)],
            Pentomino::I => vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)],
            Pentomino::L => vec![(0, 0), (0, 1), (0, 2), (0, 3), (1, 3)],
            Pentomino::N => vec![(0, 0), (1, 0), (1, 1), (2, 1), (3, 1)],
            Pentomino::P => vec![(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)],
            Pentomino::T => vec![(0, 0), (1, 0), (2, 0), (1, 1), (1, 2)],
            Pentomino::U => vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 2)],
            Pentomino::V => vec![(0, 0), (0, 1), (0, 2), (1, 2), (2, 2)],
            Pentomino::W => vec![(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)],
            Pentomino::X => vec![(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)],
            Pentomino::Y => vec![(0, 0), (1, 0), (2, 0), (3, 0), (1, 1)],
            Pentomino::Z => vec![(0, 0), (1, 0), (1, 1), (1, 2), (2, 2)],
        }
    }

    pub fn all() -> Vec<Pentomino> {
        // Try flexible pieces first
        vec![
            Pentomino::F, Pentomino::L, Pentomino::N, Pentomino::P,
            Pentomino::Y, Pentomino::Z, Pentomino::T, Pentomino::U,
            Pentomino::V, Pentomino::W, Pentomino::I, Pentomino::X,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Placement {
    pub pentomino: Pentomino,
    pub cells: Vec<(usize, usize)>,
}

pub fn generate_orientations(shape: &[(i32, i32)]) -> Vec<Vec<(i32, i32)>> {
    let mut unique = HashSet::new();
    let mut orientations = Vec::new();

    // Try all 4 rotations
    let mut current = shape.to_vec();
    for _ in 0..4 {
        let normalized = normalize(&current);
        if unique.insert(normalized.clone()) {
            orientations.push(normalized);
        }
        current = rotate_90(&current);
    }

    // Try all 4 rotations of the mirrored shape
    current = mirror(&shape.to_vec());
    for _ in 0..4 {
        let normalized = normalize(&current);
        if unique.insert(normalized.clone()) {
            orientations.push(normalized);
        }
        current = rotate_90(&current);
    }

    orientations
}

fn rotate_90(shape: &[(i32, i32)]) -> Vec<(i32, i32)> {
    shape.iter().map(|&(x, y)| (-y, x)).collect()
}

fn mirror(shape: &[(i32, i32)]) -> Vec<(i32, i32)> {
    shape.iter().map(|&(x, y)| (-x, y)).collect()
}

fn normalize(shape: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if shape.is_empty() {
        return Vec::new();
    }

    let min_x = shape.iter().map(|&(x, _)| x).min().unwrap();
    let min_y = shape.iter().map(|&(_, y)| y).min().unwrap();

    let mut normalized: Vec<(i32, i32)> = shape
        .iter()
        .map(|&(x, y)| (x - min_x, y - min_y))
        .collect();

    normalized.sort();
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_pentominoes_have_5_cells() {
        for p in Pentomino::all() {
            assert_eq!(p.base_shape().len(), 5, "{:?} should have 5 cells", p);
        }
    }

    #[test]
    fn test_generate_orientations() {
        let shape = vec![(0, 0), (1, 0), (0, 1)];
        let orientations = generate_orientations(&shape);
        assert!(orientations.len() > 0);
        assert!(orientations.len() <= 8);
    }
}
