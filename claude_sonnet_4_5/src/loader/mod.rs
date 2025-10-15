// Game loader module
// Handles loading games from textual input files

use crate::data_model::*;
use std::fs;
use std::path::Path;

pub fn load_game<P: AsRef<Path>>(path: P) -> Result<Game, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    parse_game(&content)
}

fn parse_game(content: &str) -> Result<Game, String> {
    let mut lines = content.lines().peekable();

    // Skip optional comment line
    if let Some(line) = lines.peek() {
        if line.trim().starts_with("//") {
            lines.next();
        }
    }

    // Parse board
    let board = parse_board(&mut lines)?;

    // Parse pieces
    let pieces = parse_pieces(&mut lines)?;

    // Parse constraints
    let constraints = parse_constraints(&mut lines)?;

    let game = Game::new(board, pieces, constraints);

    // Validate the game
    if !game.is_valid() {
        return Err("Game is invalid: board size must equal 2 * number of pieces, and constraints must be consistent".to_string());
    }

    Ok(game)
}

fn parse_board(lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<Board, String> {
    // Expect "board:" line
    let header = lines
        .next()
        .ok_or("Expected 'board:' header")?
        .trim();
    if header != "board:" {
        return Err(format!("Expected 'board:', got '{}'", header));
    }

    let mut points = std::collections::HashSet::new();
    let mut y = 0;

    loop {
        if let Some(line) = lines.peek() {
            if line.trim().is_empty() {
                lines.next(); // consume blank line
                break;
            }
        } else {
            break;
        }

        let line = lines.next().unwrap();
        for (x, ch) in line.chars().enumerate() {
            if ch == '#' {
                points.insert(Point::new(x, y));
            }
        }
        y += 1;
    }

    Ok(Board::new(points))
}

fn parse_pieces(lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<Vec<Piece>, String> {
    // Expect "pieces:" line
    let header = lines
        .next()
        .ok_or("Expected 'pieces:' header")?
        .trim();
    if header != "pieces:" {
        return Err(format!("Expected 'pieces:', got '{}'", header));
    }

    // Read piece line
    let pieces_line = lines
        .next()
        .ok_or("Expected pieces data")?
        .trim();

    let mut pieces = Vec::new();
    if !pieces_line.is_empty() {
        for piece_str in pieces_line.split(',') {
            let piece_str = piece_str.trim();
            if piece_str.len() != 2 {
                return Err(format!("Invalid piece format: '{}', expected 2 digits", piece_str));
            }

            let chars: Vec<char> = piece_str.chars().collect();
            let p1 = chars[0]
                .to_digit(10)
                .ok_or(format!("Invalid digit: '{}'", chars[0]))?;
            let p2 = chars[1]
                .to_digit(10)
                .ok_or(format!("Invalid digit: '{}'", chars[1]))?;

            let pips1 = Pips::new(p1 as u8)?;
            let pips2 = Pips::new(p2 as u8)?;
            pieces.push(Piece::new(pips1, pips2));
        }
    }

    // Consume blank line
    if let Some(line) = lines.peek() {
        if line.trim().is_empty() {
            lines.next();
        }
    }

    Ok(pieces)
}

fn parse_constraints(lines: &mut std::iter::Peekable<std::str::Lines>) -> Result<Vec<Constraint>, String> {
    // Expect "constraints:" line
    let header = lines
        .next()
        .ok_or("Expected 'constraints:' header")?
        .trim();
    if header != "constraints:" {
        return Err(format!("Expected 'constraints:', got '{}'", header));
    }

    let mut constraints = Vec::new();

    loop {
        if let Some(line) = lines.peek() {
            if line.trim().is_empty() {
                lines.next(); // consume blank line
                break;
            }
        } else {
            break;
        }

        let line = lines.next().unwrap().trim();
        if line.is_empty() {
            break;
        }

        constraints.push(parse_constraint(line)?);
    }

    Ok(constraints)
}

fn parse_constraint(line: &str) -> Result<Constraint, String> {
    // Format: <type> <arg?> {<points>}
    // Special case for AllDifferent: AllDifferent {} {<points>}

    let parts: Vec<&str> = line.split('{').collect();

    if parts.is_empty() {
        return Err(format!("Invalid constraint format: '{}'", line));
    }

    let header = parts[0].trim();

    // Parse constraint type and argument
    let header_parts: Vec<&str> = header.split_whitespace().collect();
    if header_parts.is_empty() {
        return Err(format!("Invalid constraint header: '{}'", header));
    }

    let constraint_type = header_parts[0];

    match constraint_type {
        "AllSame" => {
            if parts.len() != 2 {
                return Err(format!("Invalid AllSame constraint format: '{}'", line));
            }
            let points_str = parts[1].trim().trim_end_matches('}').trim();
            let points = parse_points(points_str)?;

            let target = if header_parts.len() > 1 {
                if header_parts[1] == "None" {
                    None
                } else {
                    let val: u8 = header_parts[1]
                        .parse()
                        .map_err(|_| format!("Invalid pips value: '{}'", header_parts[1]))?;
                    Some(Pips::new(val)?)
                }
            } else {
                None
            };
            Constraint::all_same(target, points)
        }
        "AllDifferent" => {
            // Format: AllDifferent {} {<points>}
            if parts.len() != 3 {
                return Err(format!("Invalid AllDifferent constraint format: '{}'", line));
            }
            // parts[1] is the excluded set (always {} in examples)
            // parts[2] is the points set
            let points_str = parts[2].trim().trim_end_matches('}').trim();
            let points = parse_points(points_str)?;

            let excluded = std::collections::HashSet::new();
            Constraint::all_different(excluded, points)
        }
        "LessThan" => {
            if parts.len() != 2 {
                return Err(format!("Invalid LessThan constraint format: '{}'", line));
            }
            if header_parts.len() < 2 {
                return Err("LessThan requires a target value".to_string());
            }
            let points_str = parts[1].trim().trim_end_matches('}').trim();
            let points = parse_points(points_str)?;
            let target: usize = header_parts[1]
                .parse()
                .map_err(|_| format!("Invalid target value: '{}'", header_parts[1]))?;
            Constraint::less_than(target, points)
        }
        "Exactly" => {
            if parts.len() != 2 {
                return Err(format!("Invalid Exactly constraint format: '{}'", line));
            }
            if header_parts.len() < 2 {
                return Err("Exactly requires a target value".to_string());
            }
            let points_str = parts[1].trim().trim_end_matches('}').trim();
            let points = parse_points(points_str)?;
            let target: usize = header_parts[1]
                .parse()
                .map_err(|_| format!("Invalid target value: '{}'", header_parts[1]))?;
            Constraint::exactly(target, points)
        }
        "MoreThan" => {
            if parts.len() != 2 {
                return Err(format!("Invalid MoreThan constraint format: '{}'", line));
            }
            if header_parts.len() < 2 {
                return Err("MoreThan requires a target value".to_string());
            }
            let points_str = parts[1].trim().trim_end_matches('}').trim();
            let points = parse_points(points_str)?;
            let target: usize = header_parts[1]
                .parse()
                .map_err(|_| format!("Invalid target value: '{}'", header_parts[1]))?;
            Constraint::more_than(target, points)
        }
        _ => Err(format!("Unknown constraint type: '{}'", constraint_type)),
    }
}

fn parse_points(points_str: &str) -> Result<std::collections::HashSet<Point>, String> {
    let mut points = std::collections::HashSet::new();

    if points_str.is_empty() {
        return Ok(points);
    }

    // Split by ")," to separate points, keeping the closing paren with each point
    let point_strs: Vec<&str> = points_str.split("),").collect();

    for (i, point_str) in point_strs.iter().enumerate() {
        // Re-add closing paren if not the last item
        let point_str = if i < point_strs.len() - 1 {
            format!("{})", point_str)
        } else {
            point_str.to_string()
        };

        let point_str = point_str.trim();
        if !point_str.starts_with('(') || !point_str.ends_with(')') {
            return Err(format!("Invalid point format: '{}'", point_str));
        }

        let coords = &point_str[1..point_str.len() - 1];
        let coord_parts: Vec<&str> = coords.split(',').collect();
        if coord_parts.len() != 2 {
            return Err(format!("Invalid point coordinates: '{}'", point_str));
        }

        let x: usize = coord_parts[0]
            .trim()
            .parse()
            .map_err(|_| format!("Invalid x coordinate: '{}'", coord_parts[0]))?;
        let y: usize = coord_parts[1]
            .trim()
            .parse()
            .map_err(|_| format!("Invalid y coordinate: '{}'", coord_parts[1]))?;

        points.insert(Point::new(x, y));
    }

    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_easy_puzzle() {
        // Test loading one of the puzzle files
        let game = load_game("../examples/easy_2025-10-13_puzzle.txt");
        assert!(game.is_ok(), "Failed to load puzzle: {:?}", game.err());

        let game = game.unwrap();
        assert_eq!(game.pieces.len(), 6);
        assert_eq!(game.board.points().len(), 12); // 6 pieces * 2 points each
        assert!(game.is_valid());
    }
}
