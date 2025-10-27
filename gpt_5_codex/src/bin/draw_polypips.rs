use pips_solver::display;
use pips_solver::model::{Board, Game, Piece, Pips, Placement, Point, PolyShape};
use std::collections::HashSet;
use std::env;
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let mut compact = false;
    let mut tokens = Vec::new();

    while let Some(arg) = args.next() {
        if arg == "--compact" {
            compact = true;
        } else if arg.starts_with('-') {
            return Err(format!("Unknown flag '{}'.", arg));
        } else {
            tokens.push(arg);
        }
    }

    if tokens.is_empty() {
        return Err("Usage: draw-polypips [--compact] <piece-token> [...]\n\
             Example: draw-polypips 5Z-:12345"
            .to_string());
    }

    for (index, token) in tokens.iter().enumerate() {
        let parsed = parse_piece_token(token)?;
        let lines = render_piece(&parsed, compact)?;
        for line in lines {
            println!("{}", line);
        }
        if index + 1 != tokens.len() {
            println!();
        }
    }

    Ok(())
}

struct ParsedPiece {
    piece: Piece,
    pip_order: Vec<Pips>,
    rotation_angle: u16,
}

fn parse_piece_token(token: &str) -> Result<ParsedPiece, String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Err("Empty piece token encountered.".to_string());
    }

    let parts: Vec<&str> = trimmed.split(':').collect();
    let (code_part, digits_part, angle_part) = match parts.len() {
        1 if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() == 2 => {
            ("2I", trimmed, None)
        }
        2 => (parts[0], parts[1], None),
        3 => (parts[0], parts[1], Some(parts[2])),
        _ => {
            return Err(format!(
                "Piece token '{}' must be of the form shape:pips[:rotation] (e.g., 5Z-:12345:90).",
                token
            ));
        }
    };

    let shape = PolyShape::from_code(code_part.trim())
        .ok_or_else(|| format!("Unknown shape code '{}'.", code_part))?;

    let digits: Vec<char> = digits_part.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != shape.cell_count() {
        return Err(format!(
            "Piece {} requires {} digits, got {} (from '{}').",
            shape.code(),
            shape.cell_count(),
            digits.len(),
            digits_part
        ));
    }

    let mut pips = Vec::with_capacity(digits.len());
    for ch in digits {
        let value = ch.to_digit(10).unwrap() as u8;
        pips.push(Pips::new(value)?);
    }

    let piece = Piece::new(shape, pips.clone())
        .map_err(|err| format!("Failed to construct piece: {}", err))?;

    let angle = match angle_part {
        None => 0,
        Some(value) => parse_rotation(value)?,
    };

    Ok(ParsedPiece {
        piece,
        pip_order: pips,
        rotation_angle: angle,
    })
}

fn parse_rotation(value: &str) -> Result<u16, String> {
    match value.trim() {
        "0" | "" => Ok(0),
        "90" => Ok(90),
        "180" => Ok(180),
        "270" => Ok(270),
        other => Err(format!(
            "Unsupported rotation '{}'. Use 0, 90, 180, or 270.",
            other
        )),
    }
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

fn render_piece(parsed: &ParsedPiece, compact: bool) -> Result<Vec<String>, String> {
    let base_offsets = parsed.piece.orientations()[0].clone();
    let mut rotated_cells: Vec<(i32, i32, Pips)> = base_offsets
        .iter()
        .zip(parsed.pip_order.iter())
        .map(|((x, y), pip)| {
            let (rx, ry) = rotate_point(*x, *y, parsed.rotation_angle);
            (rx, ry, *pip)
        })
        .collect();

    let min_x = rotated_cells.iter().map(|(x, _, _)| *x).min().unwrap_or(0);
    let min_y = rotated_cells.iter().map(|(_, y, _)| *y).min().unwrap_or(0);
    for (x, y, _) in rotated_cells.iter_mut() {
        *x -= min_x;
        *y -= min_y;
    }

    let rotated_offsets: Vec<(i32, i32)> = rotated_cells.iter().map(|(x, y, _)| (*x, *y)).collect();

    let orientation_index = parsed
        .piece
        .orientations()
        .iter()
        .position(|orientation| orientation == &rotated_offsets)
        .unwrap_or(
            parsed
                .piece
                .orientation_index_for_angle(parsed.rotation_angle),
        );

    let orientation_offsets = parsed.piece.orientations()[orientation_index].clone();
    let mut pip_order = Vec::with_capacity(rotated_cells.len());
    for (ox, oy) in orientation_offsets.iter() {
        if let Some((_, _, pip)) = rotated_cells
            .iter()
            .find(|(rx, ry, _)| rx == ox && ry == oy)
        {
            pip_order.push(*pip);
        }
    }

    let placement = Placement::new(
        parsed.piece.clone(),
        Point::new(0, 0),
        orientation_index,
        pip_order.clone(),
    );
    let points: HashSet<Point> = placement.points().into_iter().collect();
    let board = Board::new(points.clone());
    let game = Game::new(board, vec![parsed.piece.clone()], Vec::new());

    if compact {
        Ok(render_compact(&placement))
    } else {
        Ok(display::render_solution(&game, &[placement]))
    }
}

fn render_compact(placement: &Placement) -> Vec<String> {
    let assignments = placement.assignments();
    if assignments.is_empty() {
        return Vec::new();
    }
    let min_x = assignments.iter().map(|a| a.point.x).min().unwrap();
    let max_x = assignments.iter().map(|a| a.point.x).max().unwrap();
    let min_y = assignments.iter().map(|a| a.point.y).min().unwrap();
    let max_y = assignments.iter().map(|a| a.point.y).max().unwrap();

    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;
    let mut grid = vec![vec![' '; width]; height];

    for assignment in assignments {
        let x = (assignment.point.x - min_x) as usize;
        let y = (assignment.point.y - min_y) as usize;
        grid[y][x] = std::char::from_digit(assignment.pips.value() as u32, 10).unwrap();
    }

    grid.into_iter()
        .map(|row| {
            let line: String = row.into_iter().collect();
            line.trim_end().to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rotation_suffix() {
        let parsed = parse_piece_token("4L+:0123:90").expect("should parse");
        let expected = parsed.piece.orientation_index_for_angle(90);
        assert_eq!(
            parsed
                .piece
                .orientation_index_for_angle(parsed.rotation_angle),
            expected
        );
    }
}
