use crate::model::piece::PolyShape;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum PieceRule {
    Unlimited(Vec<PolyShape>),
    Exact(Vec<PolyShape>),
    ExactPentominoSet,
}

#[derive(Clone, Debug)]
pub enum ConstraintRule {
    None,
    Allowed(Vec<PolyShape>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstraintSelection {
    UniformAll,
    UniformSize,
}

pub fn parse_piece_rule(value: &str) -> Result<PieceRule, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("pieces rule must not be empty.".to_string());
    }

    let tokens: Vec<&str> = value
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() {
        return Err("pieces rule must contain at least one token.".to_string());
    }

    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("12x5") {
        return Ok(PieceRule::ExactPentominoSet);
    }
    if tokens
        .iter()
        .any(|token| token.eq_ignore_ascii_case("12x5"))
    {
        return Err("12x5 cannot be combined with other piece rules.".to_string());
    }

    let mut shapes = HashSet::new();
    for token in tokens {
        let parsed = parse_shape_token(token, ShapeContext::Pieces)?;
        shapes.extend(parsed);
    }

    if shapes.is_empty() {
        return Err("pieces rule did not resolve to any shapes.".to_string());
    }
    let mut collected: Vec<PolyShape> = shapes.into_iter().collect();
    collected.sort_by_key(|shape| shape.code().to_string());
    Ok(PieceRule::Unlimited(collected))
}

pub fn parse_constraint_rule(value: &str) -> Result<ConstraintRule, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("constraints rule must not be empty.".to_string());
    }
    if value.eq_ignore_ascii_case("none") {
        return Ok(ConstraintRule::None);
    }

    let tokens: Vec<&str> = value
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() {
        return Err("constraints rule must contain at least one token.".to_string());
    }

    let mut shapes = HashSet::new();
    for token in tokens {
        let parsed = parse_shape_token(token, ShapeContext::Constraints)?;
        shapes.extend(parsed);
    }
    if shapes.is_empty() {
        return Err("constraints rule did not resolve to any shapes.".to_string());
    }
    let mut collected: Vec<PolyShape> = shapes.into_iter().collect();
    collected.sort_by_key(|shape| shape.code().to_string());
    Ok(ConstraintRule::Allowed(collected))
}

pub fn parse_constraint_selection(value: Option<&str>) -> Result<ConstraintSelection, String> {
    match value {
        None => Ok(ConstraintSelection::UniformAll),
        Some(raw) if raw.trim().is_empty() => Ok(ConstraintSelection::UniformAll),
        Some(raw) => {
            let normalized = raw.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "uniform-all" => Ok(ConstraintSelection::UniformAll),
                "uniform-size" => Ok(ConstraintSelection::UniformSize),
                _ => Err(format!(
                    "Unknown constraint-selection '{}'. Expected 'uniform-all' or 'uniform-size'.",
                    raw
                )),
            }
        }
    }
}

pub fn all_shapes() -> &'static [PolyShape] {
    &ALL_SHAPES
}

pub fn shapes_for_size(size: usize) -> Result<&'static [PolyShape], String> {
    match size {
        1 => Ok(&SHAPES_1),
        2 => Ok(&SHAPES_2),
        3 => Ok(&SHAPES_3),
        4 => Ok(&SHAPES_4),
        5 => Ok(&SHAPES_5),
        other => Err(format!("No shapes defined for size {}.", other)),
    }
}

#[derive(Clone, Copy)]
enum ShapeContext {
    Pieces,
    Constraints,
}

fn parse_shape_token(token: &str, context: ShapeContext) -> Result<Vec<PolyShape>, String> {
    if token.is_empty() {
        return Err("Encountered empty shape token.".to_string());
    }
    if token.eq_ignore_ascii_case("any") {
        return Ok(all_shapes().to_vec());
    }

    let token = token.trim().to_ascii_uppercase();
    if token == "NONE" {
        if matches!(context, ShapeContext::Constraints) {
            return Ok(Vec::new());
        } else {
            return Err("pieces rule does not support 'none'.".to_string());
        }
    }

    if token.ends_with('*') {
        let digits = &token[..token.len() - 1];
        if digits.is_empty() {
            return Err("Shape wildcard must include a size prefix (e.g., '4*').".to_string());
        }
        let size: usize = digits
            .parse()
            .map_err(|_| format!("Invalid wildcard size '{}'.", digits))?;
        return shapes_for_size(size).map(|slice| slice.to_vec());
    }

    if let Some(shapes) = parse_numbered_shape(&token)? {
        return Ok(shapes);
    }

    Err(format!("Unrecognized shape token '{}'.", token))
}

fn parse_numbered_shape(token: &str) -> Result<Option<Vec<PolyShape>>, String> {
    let mut idx = 0;
    let chars: Vec<char> = token.chars().collect();
    while idx < chars.len() && chars[idx].is_ascii_digit() {
        idx += 1;
    }

    let (size, remainder) = if idx == 0 {
        // No explicit size; assume pentomino notation.
        (5usize, token)
    } else {
        let size: usize = token[..idx]
            .parse()
            .map_err(|_| format!("Invalid shape size in '{}'.", token))?;
        (size, &token[idx..])
    };

    if remainder.is_empty() {
        return shapes_for_size(size).map(|slice| Some(slice.to_vec()));
    }

    let normalized = remainder.to_string();
    let shapes = match size {
        1 => parse_shape_family(size, &normalized, &[("O", &SHAPES_1[..])]),
        2 => parse_shape_family(size, &normalized, &[("I", &SHAPES_2[..])]),
        3 => parse_shape_family(
            size,
            &normalized,
            &[("I", &SHAPES_3[..1]), ("L", &SHAPES_3[1..])],
        ),
        4 => parse_shape_family(
            size,
            &normalized,
            &[
                ("I", slice_single(PolyShape::TetI)),
                ("L", &[PolyShape::TetLPlus, PolyShape::TetLMinus]),
                ("L+", slice_single(PolyShape::TetLPlus)),
                ("L-", slice_single(PolyShape::TetLMinus)),
                ("O", slice_single(PolyShape::TetO)),
                ("T", slice_single(PolyShape::TetT)),
                ("S", &[PolyShape::TetSPlus, PolyShape::TetSMinus]),
                ("S+", slice_single(PolyShape::TetSPlus)),
                ("S-", slice_single(PolyShape::TetSMinus)),
            ],
        ),
        5 => parse_shape_family(
            size,
            &normalized,
            &[
                ("F", &[PolyShape::PentFPlus, PolyShape::PentFMinus]),
                ("F+", slice_single(PolyShape::PentFPlus)),
                ("F-", slice_single(PolyShape::PentFMinus)),
                ("I", slice_single(PolyShape::PentI)),
                ("L", &[PolyShape::PentLPlus, PolyShape::PentLMinus]),
                ("L+", slice_single(PolyShape::PentLPlus)),
                ("L-", slice_single(PolyShape::PentLMinus)),
                ("P", &[PolyShape::PentPPlus, PolyShape::PentPMinus]),
                ("P+", slice_single(PolyShape::PentPPlus)),
                ("P-", slice_single(PolyShape::PentPMinus)),
                ("N", &[PolyShape::PentNPlus, PolyShape::PentNMinus]),
                ("N+", slice_single(PolyShape::PentNPlus)),
                ("N-", slice_single(PolyShape::PentNMinus)),
                ("T", slice_single(PolyShape::PentT)),
                ("U", slice_single(PolyShape::PentU)),
                ("V", slice_single(PolyShape::PentV)),
                ("W", slice_single(PolyShape::PentW)),
                ("X", slice_single(PolyShape::PentX)),
                ("Y", &[PolyShape::PentYPlus, PolyShape::PentYMinus]),
                ("Y+", slice_single(PolyShape::PentYPlus)),
                ("Y-", slice_single(PolyShape::PentYMinus)),
                ("Z", &[PolyShape::PentZPlus, PolyShape::PentZMinus]),
                ("Z+", slice_single(PolyShape::PentZPlus)),
                ("Z-", slice_single(PolyShape::PentZMinus)),
            ],
        ),
        _ => Err(format!("No shapes defined for size {}.", size)),
    }?;

    Ok(Some(shapes))
}

fn parse_shape_family(
    size: usize,
    token: &str,
    families: &[(&str, &[PolyShape])],
) -> Result<Vec<PolyShape>, String> {
    for (label, shapes) in families {
        if token == *label {
            return Ok(shapes.to_vec());
        }
    }
    Err(format!("Unknown shape '{}{}'.", size, token))
}

fn slice_single(shape: PolyShape) -> &'static [PolyShape] {
    match shape {
        PolyShape::Mono => &SHAPES_1,
        PolyShape::Domino => &SHAPES_2,
        PolyShape::TriI => &SHAPES_TRI_I,
        PolyShape::TriL => &SHAPES_TRI_L,
        PolyShape::TetI => &SHAPES_TET_I,
        PolyShape::TetLPlus => &SHAPES_TET_L_PLUS,
        PolyShape::TetLMinus => &SHAPES_TET_L_MINUS,
        PolyShape::TetO => &SHAPES_TET_O,
        PolyShape::TetSPlus => &SHAPES_TET_S_PLUS,
        PolyShape::TetSMinus => &SHAPES_TET_S_MINUS,
        PolyShape::TetT => &SHAPES_TET_T,
        PolyShape::PentFPlus => &SHAPES_PENT_F_PLUS,
        PolyShape::PentFMinus => &SHAPES_PENT_F_MINUS,
        PolyShape::PentI => &SHAPES_PENT_I,
        PolyShape::PentLPlus => &SHAPES_PENT_L_PLUS,
        PolyShape::PentLMinus => &SHAPES_PENT_L_MINUS,
        PolyShape::PentPPlus => &SHAPES_PENT_P_PLUS,
        PolyShape::PentPMinus => &SHAPES_PENT_P_MINUS,
        PolyShape::PentNPlus => &SHAPES_PENT_N_PLUS,
        PolyShape::PentNMinus => &SHAPES_PENT_N_MINUS,
        PolyShape::PentT => &SHAPES_PENT_T,
        PolyShape::PentU => &SHAPES_PENT_U,
        PolyShape::PentV => &SHAPES_PENT_V,
        PolyShape::PentW => &SHAPES_PENT_W,
        PolyShape::PentX => &SHAPES_PENT_X,
        PolyShape::PentYPlus => &SHAPES_PENT_Y_PLUS,
        PolyShape::PentYMinus => &SHAPES_PENT_Y_MINUS,
        PolyShape::PentZPlus => &SHAPES_PENT_Z_PLUS,
        PolyShape::PentZMinus => &SHAPES_PENT_Z_MINUS,
    }
}

const SHAPES_1: [PolyShape; 1] = [PolyShape::Mono];
const SHAPES_2: [PolyShape; 1] = [PolyShape::Domino];
const SHAPES_TRI_I: [PolyShape; 1] = [PolyShape::TriI];
const SHAPES_TRI_L: [PolyShape; 1] = [PolyShape::TriL];
const SHAPES_3: [PolyShape; 2] = [PolyShape::TriI, PolyShape::TriL];
const SHAPES_TET_I: [PolyShape; 1] = [PolyShape::TetI];
const SHAPES_TET_L_PLUS: [PolyShape; 1] = [PolyShape::TetLPlus];
const SHAPES_TET_L_MINUS: [PolyShape; 1] = [PolyShape::TetLMinus];
const SHAPES_TET_O: [PolyShape; 1] = [PolyShape::TetO];
const SHAPES_TET_S_PLUS: [PolyShape; 1] = [PolyShape::TetSPlus];
const SHAPES_TET_S_MINUS: [PolyShape; 1] = [PolyShape::TetSMinus];
const SHAPES_TET_T: [PolyShape; 1] = [PolyShape::TetT];
const SHAPES_4: [PolyShape; 7] = [
    PolyShape::TetI,
    PolyShape::TetLPlus,
    PolyShape::TetLMinus,
    PolyShape::TetO,
    PolyShape::TetSPlus,
    PolyShape::TetSMinus,
    PolyShape::TetT,
];
const SHAPES_PENT_F_PLUS: [PolyShape; 1] = [PolyShape::PentFPlus];
const SHAPES_PENT_F_MINUS: [PolyShape; 1] = [PolyShape::PentFMinus];
const SHAPES_PENT_I: [PolyShape; 1] = [PolyShape::PentI];
const SHAPES_PENT_L_PLUS: [PolyShape; 1] = [PolyShape::PentLPlus];
const SHAPES_PENT_L_MINUS: [PolyShape; 1] = [PolyShape::PentLMinus];
const SHAPES_PENT_P_PLUS: [PolyShape; 1] = [PolyShape::PentPPlus];
const SHAPES_PENT_P_MINUS: [PolyShape; 1] = [PolyShape::PentPMinus];
const SHAPES_PENT_N_PLUS: [PolyShape; 1] = [PolyShape::PentNPlus];
const SHAPES_PENT_N_MINUS: [PolyShape; 1] = [PolyShape::PentNMinus];
const SHAPES_PENT_T: [PolyShape; 1] = [PolyShape::PentT];
const SHAPES_PENT_U: [PolyShape; 1] = [PolyShape::PentU];
const SHAPES_PENT_V: [PolyShape; 1] = [PolyShape::PentV];
const SHAPES_PENT_W: [PolyShape; 1] = [PolyShape::PentW];
const SHAPES_PENT_X: [PolyShape; 1] = [PolyShape::PentX];
const SHAPES_PENT_Y_PLUS: [PolyShape; 1] = [PolyShape::PentYPlus];
const SHAPES_PENT_Y_MINUS: [PolyShape; 1] = [PolyShape::PentYMinus];
const SHAPES_PENT_Z_PLUS: [PolyShape; 1] = [PolyShape::PentZPlus];
const SHAPES_PENT_Z_MINUS: [PolyShape; 1] = [PolyShape::PentZMinus];
const SHAPES_5: [PolyShape; 18] = [
    PolyShape::PentFPlus,
    PolyShape::PentFMinus,
    PolyShape::PentI,
    PolyShape::PentLPlus,
    PolyShape::PentLMinus,
    PolyShape::PentPPlus,
    PolyShape::PentPMinus,
    PolyShape::PentNPlus,
    PolyShape::PentNMinus,
    PolyShape::PentT,
    PolyShape::PentU,
    PolyShape::PentV,
    PolyShape::PentW,
    PolyShape::PentX,
    PolyShape::PentYPlus,
    PolyShape::PentYMinus,
    PolyShape::PentZPlus,
    PolyShape::PentZMinus,
];

const ALL_SHAPES: [PolyShape; 29] = [
    PolyShape::Mono,
    PolyShape::Domino,
    PolyShape::TriI,
    PolyShape::TriL,
    PolyShape::TetI,
    PolyShape::TetLPlus,
    PolyShape::TetLMinus,
    PolyShape::TetO,
    PolyShape::TetSPlus,
    PolyShape::TetSMinus,
    PolyShape::TetT,
    PolyShape::PentFPlus,
    PolyShape::PentFMinus,
    PolyShape::PentI,
    PolyShape::PentLPlus,
    PolyShape::PentLMinus,
    PolyShape::PentPPlus,
    PolyShape::PentPMinus,
    PolyShape::PentNPlus,
    PolyShape::PentNMinus,
    PolyShape::PentT,
    PolyShape::PentU,
    PolyShape::PentV,
    PolyShape::PentW,
    PolyShape::PentX,
    PolyShape::PentYPlus,
    PolyShape::PentYMinus,
    PolyShape::PentZPlus,
    PolyShape::PentZMinus,
];
