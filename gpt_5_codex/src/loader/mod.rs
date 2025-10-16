use crate::model::{Board, Constraint, ConstraintSet, Game, Piece, Pips, Point};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

pub fn load_game_from_path<P: AsRef<Path>>(path: P) -> Result<Game, String> {
    let file = File::open(path).map_err(|err| err.to_string())?;
    load_game_from_reader(BufReader::new(file))
}

pub fn load_game_from_reader<R: BufRead>(reader: R) -> Result<Game, String> {
    let lines: Result<Vec<String>, _> = reader.lines().collect();
    let joined = lines.map_err(|err| err.to_string())?.join("\n");
    parse_game(&joined)
}

fn parse_game(contents: &str) -> Result<Game, String> {
    let sections = ParsedSections::new(contents)?;
    let board = parse_board(&sections.board_lines)?;
    let pieces = parse_pieces(&sections.pieces_line)?;
    let constraints = parse_constraints(&sections.constraint_lines)?;
    let game = Game::new(board, pieces, constraints);
    game.validate()?;
    Ok(game)
}

struct ParsedSections {
    board_lines: Vec<String>,
    pieces_line: String,
    constraint_lines: Vec<String>,
}

impl ParsedSections {
    fn new(contents: &str) -> Result<Self, String> {
        let mut lines = contents.lines().peekable();

        while let Some(line) = lines.peek() {
            if line.trim().is_empty() {
                lines.next();
            } else {
                break;
            }
        }

        if let Some(line) = lines.peek() {
            if line.trim_start().starts_with("//") {
                lines.next();
            }
        }

        expect_header(&mut lines, "board:")?;
        let board_lines = collect_until_header(&mut lines, "pieces:");

        expect_header(&mut lines, "pieces:")?;
        let pieces_raw = collect_until_header(&mut lines, "constraints:");
        let filtered_pieces: Vec<String> = pieces_raw
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if filtered_pieces.len() > 1 {
            return Err(
                "Pieces section must contain a single line of comma-separated values.".to_string(),
            );
        }
        let pieces_line = filtered_pieces.into_iter().next().unwrap_or_default();

        expect_header(&mut lines, "constraints:")?;
        let constraint_lines = collect_until_blank(&mut lines)
            .into_iter()
            .filter(|line| !line.trim().is_empty())
            .collect();

        Ok(Self {
            board_lines,
            pieces_line,
            constraint_lines,
        })
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

fn collect_until_header<'a, I>(lines: &mut std::iter::Peekable<I>, header: &str) -> Vec<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut collected = Vec::new();
    while let Some(line) = lines.peek() {
        if line.trim().eq_ignore_ascii_case(header) {
            break;
        }
        collected.push(lines.next().unwrap().to_string());
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
    Ok(Board::new(points))
}

fn parse_pieces(line: &str) -> Result<Vec<Piece>, String> {
    if line.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut pieces = Vec::new();
    for token in line.split(',') {
        let trimmed = token.trim();
        if trimmed.len() != 2 {
            return Err(format!("Invalid piece token '{}'.", trimmed));
        }
        let a = parse_digit(trimmed.chars().nth(0).unwrap())?;
        let b = parse_digit(trimmed.chars().nth(1).unwrap())?;
        let piece = Piece::new(Pips::new(a)?, Pips::new(b)?);
        pieces.push(piece);
    }
    Ok(pieces)
}

fn parse_constraints(lines: &[String]) -> Result<ConstraintSet, String> {
    let mut constraints = Vec::new();
    for line in lines {
        constraints.push(parse_constraint(line)?);
    }
    Ok(constraints)
}

fn parse_constraint(line: &str) -> Result<Constraint, String> {
    let trimmed = line.trim();
    let brace_index = trimmed
        .rfind('{')
        .ok_or_else(|| format!("Constraint '{}' is missing point list.", line))?;
    let prefix = &trimmed[..brace_index];
    let points_str = &trimmed[brace_index + 1..];
    let points_inner = points_str
        .strip_suffix('}')
        .ok_or_else(|| format!("Constraint '{}' has an unterminated point list.", line))?;
    let points = parse_points(points_inner)?;

    let mut tokens = prefix.trim().split_whitespace();
    let kind = tokens
        .next()
        .ok_or_else(|| "Missing constraint type.".to_string())?;
    match kind {
        "AllSame" => {
            let arg = tokens
                .next()
                .ok_or_else(|| "Missing AllSame argument.".to_string())?;
            let expected = match arg {
                "None" => None,
                value => Some(parse_pips_option(value)?),
            };
            Ok(Constraint::AllSame { expected, points })
        }
        "AllDifferent" => {
            let arg = tokens
                .next()
                .ok_or_else(|| "Missing AllDifferent exclusions.".to_string())?;
            let excluded = parse_pip_set(arg)?;
            Ok(Constraint::AllDifferent { excluded, points })
        }
        "Exactly" => {
            let target = parse_u32(tokens.next(), "Exactly target")?;
            Ok(Constraint::Exactly { target, points })
        }
        "LessThan" => {
            let target = parse_u32(tokens.next(), "LessThan target")?;
            Ok(Constraint::LessThan { target, points })
        }
        "MoreThan" => {
            let target = parse_u32(tokens.next(), "MoreThan target")?;
            Ok(Constraint::MoreThan { target, points })
        }
        _ => Err(format!("Unknown constraint type '{}'.", kind)),
    }
}

fn parse_points(spec: &str) -> Result<HashSet<Point>, String> {
    let cleaned = spec.trim();
    if cleaned.is_empty() {
        return Err("Constraint must reference at least one point.".to_string());
    }
    let mut points = HashSet::new();
    for segment in cleaned.split(')') {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }
        let trimmed = trimmed.trim_start_matches(',').trim_start_matches('(');
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split(',');
        let x = parts
            .next()
            .ok_or_else(|| "Point is missing an x coordinate.".to_string())?;
        let y = parts
            .next()
            .ok_or_else(|| "Point is missing a y coordinate.".to_string())?;
        if parts.next().is_some() {
            return Err("Point has too many coordinates.".to_string());
        }
        let x_val = x
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("Invalid x coordinate '{}'.", x))?;
        let y_val = y
            .trim()
            .parse::<u32>()
            .map_err(|_| format!("Invalid y coordinate '{}'.", y))?;
        points.insert(Point::new(x_val, y_val));
    }
    Ok(points)
}

fn parse_pips_option(token: &str) -> Result<Pips, String> {
    if let Some(value) = token
        .strip_prefix("Some(")
        .and_then(|s| s.strip_suffix(')'))
    {
        Pips::from_str(value)
    } else {
        Pips::from_str(token)
    }
}

fn parse_pip_set(token: &str) -> Result<HashSet<Pips>, String> {
    let inner = token
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .ok_or_else(|| "AllDifferent exclusions must be enclosed in braces.".to_string())?;
    if inner.trim().is_empty() {
        return Ok(HashSet::new());
    }
    let mut set = HashSet::new();
    for value in inner.split(',') {
        let pips = Pips::from_str(value.trim())?;
        set.insert(pips);
    }
    Ok(set)
}

fn parse_u32(token: Option<&str>, context: &str) -> Result<u32, String> {
    let raw = token.ok_or_else(|| format!("Missing {}.", context))?;
    raw.parse::<u32>()
        .map_err(|_| format!("Invalid {} '{}'.", context, raw))
}

fn parse_digit(ch: char) -> Result<u8, String> {
    ch.to_digit(10)
        .map(|value| value as u8)
        .filter(|value| (*value as u32) <= Pips::MAX as u32)
        .ok_or_else(|| format!("Invalid pip digit '{}'.", ch))
}

#[cfg(test)]
mod tests {
    use super::parse_game;

    #[test]
    fn parses_example_game() {
        let input = r#"
// NYTimes Hard 2025-10-12
board:
   #
####
####
####
####
 #

pieces:
06,26,24,64,40,45,51,44,53

constraints:
Exactly 4 {(0,1)}
AllDifferent {} {(1,1),(0,2),(1,2),(2,2)}
Exactly 1 {(2,1)}
AllSame None {(3,0),(3,1),(3,2)}
Exactly 2 {(0,3)}
Exactly 0 {(1,3)}
Exactly 2 {(2,3)}
Exactly 5 {(3,3)}
AllDifferent {} {(0,4),(1,4),(2,4)}
Exactly 3 {(3,4)}
"#;

        let game = parse_game(input).expect("game should parse");
        assert_eq!(game.board.len(), 18);
        assert_eq!(game.pieces.len(), 9);
        assert_eq!(game.constraints.len(), 10);
    }
}
