use super::pips::Pips;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PolyShape {
    Domino,
    I3,
    I4,
    I5,
}

impl PolyShape {
    pub fn cell_count(&self) -> usize {
        match self {
            PolyShape::Domino => 2,
            PolyShape::I3 => 3,
            PolyShape::I4 => 4,
            PolyShape::I5 => 5,
        }
    }

    pub fn orientations(&self) -> &'static [Vec<(i32, i32)>] {
        match self {
            PolyShape::Domino => &DOMINO_ORIENTATIONS,
            PolyShape::I3 => &I3_ORIENTATIONS,
            PolyShape::I4 => &I4_ORIENTATIONS,
            PolyShape::I5 => &I5_ORIENTATIONS,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PolyShape::Domino => "Domino",
            PolyShape::I3 => "Triomino",
            PolyShape::I4 => "Tetromino",
            PolyShape::I5 => "Pentomino",
        }
    }
}

fn compute_orientations(cells: &[(i32, i32)]) -> Vec<Vec<(i32, i32)>> {
    let mut unique = HashSet::new();
    let mut orientations = Vec::new();

    let mut rotated = cells.to_vec();
    for _ in 0..4 {
        let signature = normalized_sorted(&rotated);
        if unique.insert(signature) {
            orientations.push(normalize_preserve_order(&rotated));
        }
        rotated = rotated.iter().map(|&(x, y)| (-y, x)).collect();
    }
    orientations
}

fn normalize_preserve_order(cells: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let min_x = cells.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let min_y = cells.iter().map(|(_, y)| *y).min().unwrap_or(0);
    cells.iter().map(|&(x, y)| (x - min_x, y - min_y)).collect()
}

fn normalized_sorted(cells: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let mut normalized = normalize_preserve_order(cells);
    normalized.sort();
    normalized
}

static DOMINO_ORIENTATIONS: Lazy<Vec<Vec<(i32, i32)>>> =
    Lazy::new(|| compute_orientations(&[(0, 0), (1, 0)]));

static I3_ORIENTATIONS: Lazy<Vec<Vec<(i32, i32)>>> =
    Lazy::new(|| compute_orientations(&[(0, 0), (1, 0), (2, 0)]));

static I4_ORIENTATIONS: Lazy<Vec<Vec<(i32, i32)>>> =
    Lazy::new(|| compute_orientations(&[(0, 0), (1, 0), (2, 0), (3, 0)]));

static I5_ORIENTATIONS: Lazy<Vec<Vec<(i32, i32)>>> =
    Lazy::new(|| compute_orientations(&[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]));

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Piece {
    shape: PolyShape,
    pips: Vec<Pips>,
}

impl Piece {
    pub fn new(shape: PolyShape, pips: Vec<Pips>) -> Result<Self, String> {
        if pips.len() != shape.cell_count() {
            return Err(format!(
                "Piece with shape {} requires {} pips, got {}.",
                shape.name(),
                shape.cell_count(),
                pips.len()
            ));
        }
        let mut sorted = pips;
        sorted.sort();
        Ok(Self {
            shape,
            pips: sorted,
        })
    }

    pub fn domino(a: Pips, b: Pips) -> Self {
        let mut values = vec![a, b];
        values.sort();
        Self {
            shape: PolyShape::Domino,
            pips: values,
        }
    }

    pub fn shape(&self) -> PolyShape {
        self.shape
    }

    pub fn pips(&self) -> &[Pips] {
        &self.pips
    }

    pub fn orientations(&self) -> &'static [Vec<(i32, i32)>] {
        self.shape.orientations()
    }

    pub fn orientation_count(&self) -> usize {
        self.shape.orientations().len()
    }

    pub fn pip_permutations(&self) -> Vec<Vec<Pips>> {
        unique_permutations(&self.pips)
    }
}

fn unique_permutations(values: &[Pips]) -> Vec<Vec<Pips>> {
    let mut sorted = values.to_vec();
    sorted.sort();
    let mut used = vec![false; sorted.len()];
    let mut current = Vec::with_capacity(sorted.len());
    let mut result = Vec::new();
    backtrack_permutations(&sorted, &mut used, &mut current, &mut result);
    result
}

fn backtrack_permutations(
    values: &[Pips],
    used: &mut [bool],
    current: &mut Vec<Pips>,
    result: &mut Vec<Vec<Pips>>,
) {
    if current.len() == values.len() {
        result.push(current.clone());
        return;
    }
    let mut previous: Option<Pips> = None;
    for (idx, &value) in values.iter().enumerate() {
        if used[idx] {
            continue;
        }
        if previous == Some(value) {
            continue;
        }
        used[idx] = true;
        current.push(value);
        backtrack_permutations(values, used, current, result);
        current.pop();
        used[idx] = false;
        previous = Some(value);
    }
}

pub fn remove_one(mut pieces: Vec<Piece>, target: &Piece) -> Result<Vec<Piece>, String> {
    if let Some(index) = pieces.iter().position(|piece| piece == target) {
        pieces.remove(index);
        Ok(pieces)
    } else {
        Err(format!(
            "Piece {:?} was not present in the list of pieces.",
            target
        ))
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values: Vec<String> = self.pips.iter().map(|p| p.value().to_string()).collect();
        write!(f, "{} [{}]", self.shape.name(), values.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::{Piece, PolyShape, remove_one};
    use crate::model::pips::Pips;

    #[test]
    fn creates_domino_in_sorted_order() {
        let a = Pips::new(2).unwrap();
        let b = Pips::new(1).unwrap();
        let piece = Piece::domino(a, b);
        assert_eq!(piece.pips()[0].value(), 1);
        assert_eq!(piece.pips()[1].value(), 2);
    }

    #[test]
    fn remove_one_removes_single_occurrence() {
        let a = Piece::domino(Pips::new(1).unwrap(), Pips::new(2).unwrap());
        let b = Piece::domino(Pips::new(3).unwrap(), Pips::new(4).unwrap());
        let pieces = vec![a.clone(), a.clone(), b.clone()];

        let remaining = remove_one(pieces, &a).unwrap();
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0], a);
        assert_eq!(remaining[1], b);
    }

    #[test]
    fn remove_one_errors_when_missing() {
        let a = Piece::domino(Pips::new(1).unwrap(), Pips::new(2).unwrap());
        let pieces = vec![a.clone()];
        let b = Piece::domino(Pips::new(3).unwrap(), Pips::new(4).unwrap());
        assert!(remove_one(pieces, &b).is_err());
    }

    #[test]
    fn shape_has_two_orientations_for_line() {
        let orientations = PolyShape::I3.orientations();
        assert_eq!(orientations.len(), 2);
        assert_eq!(orientations[0].len(), 3);
    }

    #[test]
    fn piece_new_rejects_wrong_length() {
        let result = Piece::new(PolyShape::I4, vec![Pips::new(1).unwrap(); 3]);
        assert!(result.is_err());
    }

    #[test]
    fn piece_new_accepts_correct_length() {
        let result = Piece::new(PolyShape::I4, vec![Pips::new(1).unwrap(); 4]);
        assert!(result.is_ok());
    }
}
