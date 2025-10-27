use crate::model::{Board, Point};
use crate::polypips::rules::{
    ConstraintRule, ConstraintSelection, PieceRule, parse_constraint_rule,
    parse_constraint_selection, parse_piece_rule,
};
use std::collections::HashSet;

pub struct GeneratorConfig {
    pub board: Board,
    pub piece_rule: PieceRule,
    pub constraint_rule: ConstraintRule,
    pub coverage: f64,
    pub selection: ConstraintSelection,
    pub seed: Option<u64>,
}

pub fn parse_config(contents: &str) -> Result<GeneratorConfig, String> {
    let mut lines = contents.lines().peekable();

    skip_blanks(&mut lines);
    expect_header(&mut lines, "board:")?;
    let board_lines = collect_until_blank(&mut lines);
    if board_lines.is_empty() {
        return Err("Board section must contain at least one line.".to_string());
    }
    let board = parse_board(&board_lines)?;

    skip_blanks(&mut lines);

    let mut pieces_raw: Option<String> = None;
    let mut constraints_raw: Option<String> = None;
    let mut coverage: Option<f64> = None;
    let mut selection: Option<String> = None;
    let mut seed: Option<u64> = None;

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed
            .split_once(':')
            .ok_or_else(|| format!("Configuration line '{}' is missing a ':'.", trimmed))?;
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        match key.as_str() {
            "pieces" => pieces_raw = Some(value.to_string()),
            "constraints" => constraints_raw = Some(value.to_string()),
            "constraint-coverage" => {
                let parsed: f64 = value
                    .parse()
                    .map_err(|_| format!("Invalid constraint-coverage '{}'.", value))?;
                if !(0.0..=1.0).contains(&parsed) {
                    return Err("constraint-coverage must be between 0.0 and 1.0.".to_string());
                }
                coverage = Some(parsed);
            }
            "constraint-selection" => selection = Some(value.to_string()),
            "seed" => {
                let parsed: u64 = value
                    .parse()
                    .map_err(|_| format!("Invalid seed '{}'. Expected an integer.", value))?;
                seed = Some(parsed);
            }
            other => {
                return Err(format!(
                    "Unknown configuration key '{}'. Expected pieces, constraints, constraint-coverage, constraint-selection, or seed.",
                    other
                ));
            }
        }
    }

    let pieces_raw = pieces_raw.ok_or_else(|| "Missing pieces rule.".to_string())?;
    let piece_rule = parse_piece_rule(&pieces_raw)?;

    let constraint_rule = match constraints_raw {
        None => ConstraintRule::None,
        Some(raw) => parse_constraint_rule(&raw)?,
    };

    let selection = parse_constraint_selection(selection.as_deref())?;

    let coverage = coverage.unwrap_or(0.0);
    if matches!(constraint_rule, ConstraintRule::None) && coverage > 0.0 {
        return Err(
            "constraint-coverage > 0 specified but constraints rule is 'none'.".to_string(),
        );
    }

    Ok(GeneratorConfig {
        board,
        piece_rule,
        constraint_rule,
        coverage,
        selection,
        seed,
    })
}

fn skip_blanks<'a, I>(lines: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = &'a str>,
{
    while let Some(line) = lines.peek() {
        if line.trim().is_empty() {
            lines.next();
        } else {
            break;
        }
    }
}

fn expect_header<'a, I>(lines: &mut std::iter::Peekable<I>, expected: &str) -> Result<(), String>
where
    I: Iterator<Item = &'a str>,
{
    while let Some(line) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }
        return if line.trim().eq_ignore_ascii_case(expected) {
            Ok(())
        } else {
            Err(format!("Expected {} header.", expected))
        };
    }
    Err(format!("Missing {} header.", expected))
}

fn collect_until_blank<'a, I>(lines: &mut std::iter::Peekable<I>) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut collected = Vec::new();
    while let Some(line) = lines.peek() {
        if line.trim().is_empty() {
            lines.next();
            break;
        } else {
            collected.push(lines.next().unwrap().to_string());
        }
    }
    collected
}

fn parse_board(lines: &[String]) -> Result<Board, String> {
    let mut points = HashSet::new();
    for (y, row) in lines.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            match ch {
                '#' => {
                    points.insert(Point::new(x as u32, y as u32));
                }
                ' ' => {}
                _ => {
                    return Err(format!("Invalid character '{}' in board definition.", ch));
                }
            }
        }
    }
    if points.is_empty() {
        return Err("Board definition must contain at least one '#'-marked cell.".to_string());
    }
    Ok(Board::new(points))
}
