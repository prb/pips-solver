use crate::model::board::Board;
use crate::model::constraint::Constraint;
use crate::model::game::Game;
use crate::model::piece::Piece;
use crate::model::pips::Pips;
use crate::model::point::Point;
use std::collections::HashSet;
use std::fs;

pub fn load_game(file_path: &str) -> Result<Game, String> {
    let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let mut lines = content.lines();

    // Skip comment lines
    let mut non_comment_lines = lines.by_ref().skip_while(|l| l.starts_with("//"));

    let board = parse_board(&mut non_comment_lines)?;
    let pieces = parse_pieces(&mut non_comment_lines)?;
    let constraints = parse_constraints(&mut non_comment_lines)?;

    let game = Game::new(board, pieces, constraints);

    if game.is_valid() {
        Ok(game)
    } else {
        Err("Invalid game configuration".to_string())
    }
}

fn parse_board<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
) -> Result<Board, String> {
    let mut points = HashSet::new();

    let line = lines.next().ok_or("Missing board section")?;
    if line != "board:" {
        return Err("Missing 'board:' header".to_string());
    }

    for (y, line) in lines.enumerate() {
        if line.is_empty() {
            break;
        }
        for (x, c) in line.chars().enumerate() {
            if c == '#' {
                points.insert(Point(x, y));
            }
        }
    }

    Ok(Board::new(points))
}

fn parse_pieces<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Result<Vec<Piece>, String> {
    let mut pieces = Vec::new();

    let line = lines.next().ok_or("Missing pieces section")?;
    if line != "pieces:" {
        return Err("Missing 'pieces:' header".to_string());
    }

    let line = lines.next().ok_or("Missing pieces data")?;
    for piece_str in line.split(',') {
        let mut chars = piece_str.chars();
        let p1 = chars
            .next()
            .and_then(|c| c.to_digit(10))
            .ok_or("Invalid piece format")? as u8;
        let p2 = chars
            .next()
            .and_then(|c| c.to_digit(10))
            .ok_or("Invalid piece format")? as u8;
        pieces.push(Piece::new(Pips::new(p1)?, Pips::new(p2)?));
    }

    // Consume the blank line
    lines.next();

    Ok(pieces)
}

fn parse_pips_set(pips_str: &str) -> Result<HashSet<Pips>, String> {
    let mut pips = HashSet::new();
    if pips_str.starts_with("{") && pips_str.ends_with("}") {
        let inner = &pips_str[1..pips_str.len() - 1];
        if !inner.is_empty() {
            for p_str in inner.split(',') {
                let p = p_str.trim().parse::<u8>().map_err(|e| e.to_string())?;
                pips.insert(Pips::new(p)?);
            }
        }
    }
    Ok(pips)
}

use lazy_static::lazy_static;
use regex::Regex;

fn parse_constraints<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
) -> Result<Vec<Constraint>, String> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^(?P<type>\w+)\s+(?P<arg>\S+)?\s*\{(?P<points>.*)\}$").unwrap();
    }
    let mut constraints = Vec::new();

    let line = lines.next().ok_or("Missing constraints section")?;
    if line != "constraints:" {
        return Err("Missing 'constraints:' header".to_string());
    }

    for line in lines {
        if line.is_empty() {
            break;
        }
        let caps = RE.captures(line).ok_or("Invalid constraint format")?;
        let type_str = caps.name("type").unwrap().as_str();
        let arg_str = caps.name("arg").map(|m| m.as_str());
        let points_str = caps.name("points").unwrap().as_str();

        let points = parse_points(points_str)?;

        let constraint = match type_str {
            "Exactly" => {
                let val = arg_str.unwrap().parse::<usize>().unwrap();
                Constraint::new_exactly(val, points)?
            }
            "AllDifferent" => {
                let pips = parse_pips_set(arg_str.unwrap_or(""))?;
                Constraint::new_all_different(pips, points)?
            }
            "AllSame" => {
                let pips = if arg_str == Some("None") {
                    None
                } else {
                    let p = arg_str.unwrap().parse::<u8>().unwrap();
                    Some(Pips::new(p)?)
                };
                Constraint::new_all_same(pips, points)?
            }
            "LessThan" => {
                let val = arg_str.unwrap().parse::<usize>().unwrap();
                Constraint::new_less_than(val, points)?
            }
            "MoreThan" => {
                let val = arg_str.unwrap().parse::<usize>().unwrap();
                Constraint::new_more_than(val, points)?
            }
            _ => return Err(format!("Unknown constraint type: {}", type_str)),
        };
        constraints.push(constraint);
    }

    Ok(constraints)
}

fn parse_points(points_str: &str) -> Result<HashSet<Point>, String> {
    lazy_static! {
        static ref POINT_RE: Regex = Regex::new(r"\((?P<x>\d+),(?P<y>\d+)\)").unwrap();
    }
    let mut points = HashSet::new();
    for cap in POINT_RE.captures_iter(points_str) {
        let x = cap.name("x").unwrap().as_str().parse::<usize>().unwrap();
        let y = cap.name("y").unwrap().as_str().parse::<usize>().unwrap();
        points.insert(Point(x, y));
    }
    Ok(points)
}
