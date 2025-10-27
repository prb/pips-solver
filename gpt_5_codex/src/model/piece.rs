use super::pips::Pips;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

const MONO_BASE: [(i32, i32); 1] = [(0, 0)];
const DOMINO_BASE: [(i32, i32); 2] = [(0, 0), (1, 0)];
const TRI_I_BASE: [(i32, i32); 3] = [(0, 0), (1, 0), (2, 0)];
const TRI_L_BASE: [(i32, i32); 3] = [(0, 0), (0, 1), (1, 0)];
const TET_I_BASE: [(i32, i32); 4] = [(0, 0), (1, 0), (2, 0), (3, 0)];
const TET_L_BASE: [(i32, i32); 4] = [(0, 0), (0, 1), (0, 2), (1, 2)];
const TET_O_BASE: [(i32, i32); 4] = [(0, 0), (1, 0), (0, 1), (1, 1)];
const TET_S_BASE: [(i32, i32); 4] = [(0, 0), (1, 0), (1, 1), (2, 1)];
const TET_T_BASE: [(i32, i32); 4] = [(0, 0), (1, 0), (2, 0), (1, 1)];
const PENT_F_BASE: [(i32, i32); 5] = [(0, 1), (1, 0), (1, 1), (1, 2), (2, 2)];
const PENT_I_BASE: [(i32, i32); 5] = [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)];
const PENT_L_BASE: [(i32, i32); 5] = [(0, 0), (0, 1), (0, 2), (0, 3), (1, 3)];
const PENT_P_BASE: [(i32, i32); 5] = [(0, 0), (1, 0), (0, 1), (1, 1), (0, 2)];
const PENT_N_BASE: [(i32, i32); 5] = [(0, 0), (1, 0), (1, 1), (2, 1), (3, 1)];
const PENT_T_BASE: [(i32, i32); 5] = [(0, 0), (1, 0), (2, 0), (1, 1), (1, 2)];
const PENT_U_BASE: [(i32, i32); 5] = [(0, 0), (0, 1), (0, 2), (1, 0), (1, 2)];
const PENT_V_BASE: [(i32, i32); 5] = [(0, 0), (0, 1), (0, 2), (1, 2), (2, 2)];
const PENT_W_BASE: [(i32, i32); 5] = [(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)];
const PENT_X_BASE: [(i32, i32); 5] = [(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)];
const PENT_Y_BASE: [(i32, i32); 5] = [(0, 0), (1, 0), (2, 0), (3, 0), (1, 1)];
const PENT_Z_BASE: [(i32, i32); 5] = [(0, 0), (1, 0), (1, 1), (1, 2), (2, 2)];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PolyShape {
    Mono,
    Domino,
    TriI,
    TriL,
    TetI,
    TetLPlus,
    TetLMinus,
    TetO,
    TetSPlus,
    TetSMinus,
    TetT,
    PentFPlus,
    PentFMinus,
    PentI,
    PentLPlus,
    PentLMinus,
    PentPPlus,
    PentPMinus,
    PentNPlus,
    PentNMinus,
    PentT,
    PentU,
    PentV,
    PentW,
    PentX,
    PentYPlus,
    PentYMinus,
    PentZPlus,
    PentZMinus,
}

struct ShapeDescriptor {
    code: &'static str,
    name: &'static str,
    orientations: &'static Lazy<Vec<Vec<(i32, i32)>>>,
    cell_count: usize,
}

impl ShapeDescriptor {
    fn orientations(&self) -> &[Vec<(i32, i32)>] {
        self.orientations.deref()
    }
}

macro_rules! shape {
    ($desc:ident, $orient:ident, $code:expr, $name:expr, $base:expr, $reflect:expr, $count:expr) => {
        static $orient: Lazy<Vec<Vec<(i32, i32)>>> =
            Lazy::new(|| compute_orientations($base, $reflect));
        static $desc: ShapeDescriptor = ShapeDescriptor {
            code: $code,
            name: $name,
            orientations: &$orient,
            cell_count: $count,
        };
    };
}

macro_rules! shape_mirrored {
    ($desc:ident, $orient:ident, $code:expr, $name:expr, $base:expr, $count:expr) => {
        static $orient: Lazy<Vec<Vec<(i32, i32)>>> = Lazy::new(|| {
            let mirrored = mirror_cells($base);
            compute_orientations(&mirrored, false)
        });
        static $desc: ShapeDescriptor = ShapeDescriptor {
            code: $code,
            name: $name,
            orientations: &$orient,
            cell_count: $count,
        };
    };
}

shape!(
    MONO_DESCRIPTOR,
    MONO_ORIENTATIONS,
    "1O",
    "Monomino",
    &MONO_BASE,
    false,
    1
);
shape!(
    DOMINO_DESCRIPTOR,
    DOMINO_ORIENTATIONS,
    "2I",
    "Domino",
    &DOMINO_BASE,
    false,
    2
);
shape!(
    TRI_I_DESCRIPTOR,
    TRI_I_ORIENTATIONS,
    "3I",
    "Triomino I",
    &TRI_I_BASE,
    false,
    3
);
shape!(
    TRI_L_DESCRIPTOR,
    TRI_L_ORIENTATIONS,
    "3L",
    "Triomino L",
    &TRI_L_BASE,
    false,
    3
);
shape!(
    TET_I_DESCRIPTOR,
    TET_I_ORIENTATIONS,
    "4I",
    "Tetromino I",
    &TET_I_BASE,
    false,
    4
);
shape!(
    TET_L_PLUS_DESCRIPTOR,
    TET_L_PLUS_ORIENTATIONS,
    "4L+",
    "Tetromino L+",
    &TET_L_BASE,
    false,
    4
);
shape_mirrored!(
    TET_L_MINUS_DESCRIPTOR,
    TET_L_MINUS_ORIENTATIONS,
    "4L-",
    "Tetromino L-",
    &TET_L_BASE,
    4
);
shape!(
    TET_O_DESCRIPTOR,
    TET_O_ORIENTATIONS,
    "4O",
    "Tetromino O",
    &TET_O_BASE,
    false,
    4
);
shape!(
    TET_S_PLUS_DESCRIPTOR,
    TET_S_PLUS_ORIENTATIONS,
    "4S+",
    "Tetromino S+",
    &TET_S_BASE,
    false,
    4
);
shape_mirrored!(
    TET_S_MINUS_DESCRIPTOR,
    TET_S_MINUS_ORIENTATIONS,
    "4S-",
    "Tetromino S-",
    &TET_S_BASE,
    4
);
shape!(
    TET_T_DESCRIPTOR,
    TET_T_ORIENTATIONS,
    "4T",
    "Tetromino T",
    &TET_T_BASE,
    true,
    4
);
shape!(
    PENT_F_PLUS_DESCRIPTOR,
    PENT_F_PLUS_ORIENTATIONS,
    "5F+",
    "Pentomino F+",
    &PENT_F_BASE,
    false,
    5
);
shape_mirrored!(
    PENT_F_MINUS_DESCRIPTOR,
    PENT_F_MINUS_ORIENTATIONS,
    "5F-",
    "Pentomino F-",
    &PENT_F_BASE,
    5
);
shape!(
    PENT_I_DESCRIPTOR,
    PENT_I_ORIENTATIONS,
    "5I",
    "Pentomino I",
    &PENT_I_BASE,
    false,
    5
);
shape!(
    PENT_L_PLUS_DESCRIPTOR,
    PENT_L_PLUS_ORIENTATIONS,
    "5L+",
    "Pentomino L+",
    &PENT_L_BASE,
    false,
    5
);
shape_mirrored!(
    PENT_L_MINUS_DESCRIPTOR,
    PENT_L_MINUS_ORIENTATIONS,
    "5L-",
    "Pentomino L-",
    &PENT_L_BASE,
    5
);
shape!(
    PENT_P_PLUS_DESCRIPTOR,
    PENT_P_PLUS_ORIENTATIONS,
    "5P+",
    "Pentomino P+",
    &PENT_P_BASE,
    false,
    5
);
shape_mirrored!(
    PENT_P_MINUS_DESCRIPTOR,
    PENT_P_MINUS_ORIENTATIONS,
    "5P-",
    "Pentomino P-",
    &PENT_P_BASE,
    5
);
shape!(
    PENT_N_PLUS_DESCRIPTOR,
    PENT_N_PLUS_ORIENTATIONS,
    "5N+",
    "Pentomino N+",
    &PENT_N_BASE,
    false,
    5
);
shape_mirrored!(
    PENT_N_MINUS_DESCRIPTOR,
    PENT_N_MINUS_ORIENTATIONS,
    "5N-",
    "Pentomino N-",
    &PENT_N_BASE,
    5
);
shape!(
    PENT_T_DESCRIPTOR,
    PENT_T_ORIENTATIONS,
    "5T",
    "Pentomino T",
    &PENT_T_BASE,
    true,
    5
);
shape!(
    PENT_U_DESCRIPTOR,
    PENT_U_ORIENTATIONS,
    "5U",
    "Pentomino U",
    &PENT_U_BASE,
    true,
    5
);
shape!(
    PENT_V_DESCRIPTOR,
    PENT_V_ORIENTATIONS,
    "5V",
    "Pentomino V",
    &PENT_V_BASE,
    true,
    5
);
shape!(
    PENT_W_DESCRIPTOR,
    PENT_W_ORIENTATIONS,
    "5W",
    "Pentomino W",
    &PENT_W_BASE,
    true,
    5
);
shape!(
    PENT_X_DESCRIPTOR,
    PENT_X_ORIENTATIONS,
    "5X",
    "Pentomino X",
    &PENT_X_BASE,
    true,
    5
);
shape!(
    PENT_Y_PLUS_DESCRIPTOR,
    PENT_Y_PLUS_ORIENTATIONS,
    "5Y+",
    "Pentomino Y+",
    &PENT_Y_BASE,
    false,
    5
);
shape_mirrored!(
    PENT_Y_MINUS_DESCRIPTOR,
    PENT_Y_MINUS_ORIENTATIONS,
    "5Y-",
    "Pentomino Y-",
    &PENT_Y_BASE,
    5
);
shape!(
    PENT_Z_PLUS_DESCRIPTOR,
    PENT_Z_PLUS_ORIENTATIONS,
    "5Z+",
    "Pentomino Z+",
    &PENT_Z_BASE,
    false,
    5
);
shape_mirrored!(
    PENT_Z_MINUS_DESCRIPTOR,
    PENT_Z_MINUS_ORIENTATIONS,
    "5Z-",
    "Pentomino Z-",
    &PENT_Z_BASE,
    5
);

impl PolyShape {
    fn descriptor(&self) -> &'static ShapeDescriptor {
        match self {
            PolyShape::Mono => &MONO_DESCRIPTOR,
            PolyShape::Domino => &DOMINO_DESCRIPTOR,
            PolyShape::TriI => &TRI_I_DESCRIPTOR,
            PolyShape::TriL => &TRI_L_DESCRIPTOR,
            PolyShape::TetI => &TET_I_DESCRIPTOR,
            PolyShape::TetLPlus => &TET_L_PLUS_DESCRIPTOR,
            PolyShape::TetLMinus => &TET_L_MINUS_DESCRIPTOR,
            PolyShape::TetO => &TET_O_DESCRIPTOR,
            PolyShape::TetSPlus => &TET_S_PLUS_DESCRIPTOR,
            PolyShape::TetSMinus => &TET_S_MINUS_DESCRIPTOR,
            PolyShape::TetT => &TET_T_DESCRIPTOR,
            PolyShape::PentFPlus => &PENT_F_PLUS_DESCRIPTOR,
            PolyShape::PentFMinus => &PENT_F_MINUS_DESCRIPTOR,
            PolyShape::PentI => &PENT_I_DESCRIPTOR,
            PolyShape::PentLPlus => &PENT_L_PLUS_DESCRIPTOR,
            PolyShape::PentLMinus => &PENT_L_MINUS_DESCRIPTOR,
            PolyShape::PentPPlus => &PENT_P_PLUS_DESCRIPTOR,
            PolyShape::PentPMinus => &PENT_P_MINUS_DESCRIPTOR,
            PolyShape::PentNPlus => &PENT_N_PLUS_DESCRIPTOR,
            PolyShape::PentNMinus => &PENT_N_MINUS_DESCRIPTOR,
            PolyShape::PentT => &PENT_T_DESCRIPTOR,
            PolyShape::PentU => &PENT_U_DESCRIPTOR,
            PolyShape::PentV => &PENT_V_DESCRIPTOR,
            PolyShape::PentW => &PENT_W_DESCRIPTOR,
            PolyShape::PentX => &PENT_X_DESCRIPTOR,
            PolyShape::PentYPlus => &PENT_Y_PLUS_DESCRIPTOR,
            PolyShape::PentYMinus => &PENT_Y_MINUS_DESCRIPTOR,
            PolyShape::PentZPlus => &PENT_Z_PLUS_DESCRIPTOR,
            PolyShape::PentZMinus => &PENT_Z_MINUS_DESCRIPTOR,
        }
    }

    pub fn code(&self) -> &'static str {
        self.descriptor().code
    }

    pub fn name(&self) -> &'static str {
        self.descriptor().name
    }

    pub fn cell_count(&self) -> usize {
        self.descriptor().cell_count
    }

    pub fn orientations(&self) -> &'static [Vec<(i32, i32)>] {
        self.descriptor().orientations()
    }

    pub fn from_code(code: &str) -> Option<Self> {
        let normalized = code.trim().to_ascii_uppercase();
        match normalized.as_str() {
            "1O" => Some(PolyShape::Mono),
            "2I" => Some(PolyShape::Domino),
            "3I" => Some(PolyShape::TriI),
            "3L" => Some(PolyShape::TriL),
            "4I" => Some(PolyShape::TetI),
            "4L+" => Some(PolyShape::TetLPlus),
            "4L-" => Some(PolyShape::TetLMinus),
            "4O" => Some(PolyShape::TetO),
            "4S+" => Some(PolyShape::TetSPlus),
            "4S-" => Some(PolyShape::TetSMinus),
            "4T" => Some(PolyShape::TetT),
            "5F+" => Some(PolyShape::PentFPlus),
            "5F-" => Some(PolyShape::PentFMinus),
            "5I" => Some(PolyShape::PentI),
            "5L+" => Some(PolyShape::PentLPlus),
            "5L-" => Some(PolyShape::PentLMinus),
            "5P+" => Some(PolyShape::PentPPlus),
            "5P-" => Some(PolyShape::PentPMinus),
            "5N+" => Some(PolyShape::PentNPlus),
            "5N-" => Some(PolyShape::PentNMinus),
            "5T" => Some(PolyShape::PentT),
            "5U" => Some(PolyShape::PentU),
            "5V" => Some(PolyShape::PentV),
            "5W" => Some(PolyShape::PentW),
            "5X" => Some(PolyShape::PentX),
            "5Y+" => Some(PolyShape::PentYPlus),
            "5Y-" => Some(PolyShape::PentYMinus),
            "5Z+" => Some(PolyShape::PentZPlus),
            "5Z-" => Some(PolyShape::PentZMinus),
            _ => None,
        }
    }

    pub fn preferred_orientation_index(&self) -> usize {
        let orientations = self.orientations();
        if let Some(target) = canonical_target(self) {
            if let Some((idx, _)) = orientations
                .iter()
                .enumerate()
                .find(|(_, offsets)| **offsets == target)
            {
                return idx;
            }
        }
        let mut best_index = 0usize;
        let mut best_score = OrientationScore::new(&orientations[0]);
        for (idx, offsets) in orientations.iter().enumerate().skip(1) {
            let score = OrientationScore::new(offsets);
            if score < best_score {
                best_index = idx;
                best_score = score;
            }
        }
        best_index
    }
}

fn canonical_target(shape: &PolyShape) -> Option<Vec<(i32, i32)>> {
    macro_rules! mirror_of {
        ($base:expr) => {{
            let base = $base.orientations()[0].clone();
            Some(normalize_preserve_order(&mirror_cells(&base)))
        }};
    }

    match shape {
        PolyShape::TetLPlus
        | PolyShape::TetO
        | PolyShape::TetSPlus
        | PolyShape::TetT
        | PolyShape::TetI
        | PolyShape::TriI
        | PolyShape::TriL
        | PolyShape::Mono
        | PolyShape::Domino
        | PolyShape::PentI
        | PolyShape::PentT
        | PolyShape::PentU
        | PolyShape::PentV
        | PolyShape::PentW
        | PolyShape::PentX => Some(shape.orientations()[0].clone()),
        PolyShape::TetLMinus => mirror_of!(PolyShape::TetLPlus),
        PolyShape::TetSMinus => mirror_of!(PolyShape::TetSPlus),
        PolyShape::PentFPlus
        | PolyShape::PentLPlus
        | PolyShape::PentPPlus
        | PolyShape::PentNPlus
        | PolyShape::PentYPlus
        | PolyShape::PentZPlus => Some(shape.orientations()[0].clone()),
        PolyShape::PentFMinus => mirror_of!(PolyShape::PentFPlus),
        PolyShape::PentLMinus => mirror_of!(PolyShape::PentLPlus),
        PolyShape::PentPMinus => mirror_of!(PolyShape::PentPPlus),
        PolyShape::PentNMinus => mirror_of!(PolyShape::PentNPlus),
        PolyShape::PentYMinus => mirror_of!(PolyShape::PentYPlus),
        PolyShape::PentZMinus => mirror_of!(PolyShape::PentZPlus),
    }
}

fn compute_orientations(cells: &[(i32, i32)], include_reflection: bool) -> Vec<Vec<(i32, i32)>> {
    let mut unique = HashSet::new();
    let mut orientations = Vec::new();
    let mut rotated: Vec<(i32, i32)> = cells.to_vec();
    for _ in 0..4 {
        let signature = normalized_sorted(&rotated);
        if unique.insert(signature.clone()) {
            orientations.push(normalize_preserve_order(&rotated));
        }
        if include_reflection {
            let mirrored = mirror_cells(&rotated);
            let signature = normalized_sorted(&mirrored);
            if unique.insert(signature.clone()) {
                orientations.push(normalize_preserve_order(&mirrored));
            }
        }
        rotated = rotate_cw(&rotated);
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

fn rotate_cw(cells: &[(i32, i32)]) -> Vec<(i32, i32)> {
    cells.iter().map(|&(x, y)| (y, -x)).collect()
}

fn mirror_cells(cells: &[(i32, i32)]) -> Vec<(i32, i32)> {
    cells.iter().map(|&(x, y)| (-x, y)).collect()
}

fn rotate_offsets(offsets: &[(i32, i32)], angle: u16) -> Vec<(i32, i32)> {
    offsets
        .iter()
        .map(|&(x, y)| rotate_point(x, y, angle))
        .collect()
}

fn rotate_point(x: i32, y: i32, angle: u16) -> (i32, i32) {
    match angle % 360 {
        0 => (x, y),
        90 => (y, -x),
        180 => (-x, -y),
        270 => (-y, x),
        _ => (x, y),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Piece {
    shape: PolyShape,
    pips: Arc<[Pips]>,
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
        Ok(Self {
            shape,
            pips: Arc::from(pips.into_boxed_slice()),
        })
    }

    pub fn domino(a: Pips, b: Pips) -> Self {
        Self::new(PolyShape::Domino, vec![a, b]).expect("valid domino")
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
        vec![self.pips.to_vec()]
    }

    pub fn preferred_orientation_index(&self) -> usize {
        self.shape.preferred_orientation_index()
    }

    pub fn orientation_index_for_angle(&self, angle: u16) -> usize {
        let orientations = self.shape.orientations();
        if angle % 360 == 0 {
            return 0;
        }
        let base = &orientations[0];
        let rotated = rotate_offsets(base, angle);
        let mut rotated_sorted = rotated.clone();
        rotated_sorted.sort();
        orientations
            .iter()
            .position(|orientation| {
                let mut candidate = orientation.clone();
                candidate.sort();
                candidate == rotated_sorted
            })
            .unwrap_or(0)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OrientationScore(Vec<i32>);

impl OrientationScore {
    fn new(offsets: &[(i32, i32)]) -> Self {
        let has_origin = offsets.iter().any(|&(x, y)| x == 0 && y == 0);
        let max_x = offsets.iter().map(|(x, _)| *x).max().unwrap_or(0);
        let max_y = offsets.iter().map(|(_, y)| *y).max().unwrap_or(0);
        let mut ordered: Vec<(i32, i32)> = offsets.iter().copied().collect();
        ordered.sort_by_key(|&(x, y)| (y, x));
        let mut metrics = Vec::with_capacity(3 + ordered.len());
        metrics.push(if has_origin { 0 } else { 1 });
        metrics.push(max_y);
        metrics.push(max_x);
        for (x, y) in ordered {
            metrics.push(y * 16 + x);
        }
        OrientationScore(metrics)
    }
}

pub fn remove_one(mut pieces: Vec<Piece>, target: &Piece) -> Result<Vec<Piece>, String> {
    if let Some(index) = pieces.iter().position(|piece| piece == target) {
        pieces.remove(index);
        Ok(pieces)
    } else {
        Err(format!(
            "Piece {} was not present in the list of pieces.",
            target.shape().code()
        ))
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values: Vec<String> = self.pips.iter().map(|p| p.value().to_string()).collect();
        write!(f, "{} [{}]", self.shape.code(), values.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::{Piece, PolyShape, remove_one};
    use crate::model::pips::Pips;

    #[test]
    fn creates_domino_preserves_order() {
        let a = Pips::new(2).unwrap();
        let b = Pips::new(1).unwrap();
        let piece = Piece::domino(a, b);
        assert_eq!(piece.pips()[0].value(), 2);
        assert_eq!(piece.pips()[1].value(), 1);
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
    fn orientation_index_matches_rotation() {
        let piece = Piece::new(
            PolyShape::TetLPlus,
            vec![
                Pips::new(0).unwrap(),
                Pips::new(1).unwrap(),
                Pips::new(2).unwrap(),
                Pips::new(3).unwrap(),
            ],
        )
        .unwrap();
        let count = piece.orientation_count();
        let idx0 = piece.orientation_index_for_angle(0);
        let idx90 = piece.orientation_index_for_angle(90);
        let idx180 = piece.orientation_index_for_angle(180);
        assert!(idx0 < count);
        assert!(idx90 < count);
        assert!(idx180 < count);
        assert_ne!(idx0, idx90);
        assert_ne!(idx0, idx180);
    }

    #[test]
    fn shape_has_two_orientations_for_line() {
        let orientations = PolyShape::TriI.orientations();
        assert_eq!(orientations.len(), 2);
        assert_eq!(orientations[0].len(), 3);
    }

    #[test]
    fn piece_new_rejects_wrong_length() {
        let result = Piece::new(PolyShape::TetI, vec![Pips::new(1).unwrap(); 3]);
        assert!(result.is_err());
    }

    #[test]
    fn piece_new_accepts_correct_length() {
        let result = Piece::new(PolyShape::TetI, vec![Pips::new(1).unwrap(); 4]);
        assert!(result.is_ok());
    }
}
