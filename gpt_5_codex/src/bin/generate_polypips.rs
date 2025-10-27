use pips_solver::display;
use pips_solver::model::{Board, Constraint, Game, Piece, Placement, Point};
use pips_solver::polypips::{config, generator};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let path = parse_args()?;
    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read '{}': {}", path.display(), err))?;
    let config = config::parse_config(&contents)?;
    let puzzle = generator::generate(config)?;

    let game = puzzle.as_game();
    game.validate()?;

    let board_lines = render_board(&game.board);
    println!("board:");
    for line in board_lines {
        println!("{}", line);
    }
    println!();

    println!("pieces:");
    if puzzle.pieces.is_empty() {
        println!();
    } else {
        let tokens: Vec<String> = puzzle
            .pieces
            .iter()
            .map(|piece| {
                let digits: String = piece.pips().iter().map(|p| p.value().to_string()).collect();
                format!("{}:{}", piece.shape().code(), digits)
            })
            .collect();
        println!("{}", tokens.join(","));
    }
    println!();

    println!("constraints:");
    if puzzle.constraints.is_empty() {
        println!();
    } else {
        for constraint in &puzzle.constraints {
            println!("{}", format_constraint(constraint));
        }
    }
    println!();

    println!("game:");
    let unsolved = display::render_unsolved(&game);
    if unsolved.is_empty() {
        println!();
    } else {
        for line in unsolved {
            println!("{}", line);
        }
        println!();
    }

    if !puzzle.pieces.is_empty() {
        println!("pieces (ascii):");
        for piece in &puzzle.pieces {
            let digits: String = piece.pips().iter().map(|p| p.value().to_string()).collect();
            println!("{}:{}", piece.shape().code(), digits);
            for line in render_piece_ascii(piece) {
                println!("{}", line);
            }
            println!();
        }
    }
    println!();

    println!("solution:");
    let rendered = display::render_solution(&game, &puzzle.placements);
    for line in rendered {
        println!("{}", line);
    }
    Ok(())
}

fn render_piece_ascii(piece: &Piece) -> Vec<String> {
    let orientation_index = piece.preferred_orientation_index();
    let pip_order: Vec<_> = piece.pips().to_vec();
    let placement = Placement::new(
        piece.clone(),
        Point::new(0, 0),
        orientation_index,
        pip_order.clone(),
    );
    let points: HashSet<Point> = placement.points().into_iter().collect();
    let board = Board::new(points);
    let game = Game::new(board, vec![piece.clone()], Vec::new());
    display::render_solution(&game, &[placement])
}

fn parse_args() -> Result<PathBuf, String> {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(value) if !value.starts_with('-') => PathBuf::from(value),
        _ => {
            return Err("Usage: generate-polypips <config-file>\n\
                 Generates a Polypips puzzle from the provided configuration."
                .to_string());
        }
    };
    if args.next().is_some() {
        return Err("generate-polypips expects exactly one argument.".to_string());
    }
    Ok(path)
}

fn render_board(board: &Board) -> Vec<String> {
    if board.is_empty() {
        return Vec::new();
    }
    let (min_x, max_x, min_y, max_y) = board.bounds().unwrap();

    let mut rows = Vec::new();
    for y in min_y..=max_y {
        let mut line = String::new();
        for x in min_x..=max_x {
            let point = Point::new(x, y);
            if board.contains_point(&point) {
                line.push('#');
            } else {
                line.push(' ');
            }
        }
        rows.push(line);
    }
    rows
}

fn format_constraint(constraint: &Constraint) -> String {
    match constraint {
        Constraint::AllSame { expected, points } => {
            let expectation = expected
                .map(|p| format!("Some({})", p.value()))
                .unwrap_or_else(|| "None".to_string());
            format!("AllSame {} {}", expectation, format_points(points.as_ref()))
        }
        Constraint::AllDifferent { excluded, points } => {
            let excluded_tokens: Vec<String> =
                excluded.iter().map(|p| p.value().to_string()).collect();
            format!(
                "AllDifferent {{{}}} {}",
                excluded_tokens.join(","),
                format_points(points.as_ref())
            )
        }
        Constraint::Exactly { target, points } => {
            format!("Exactly {} {}", target, format_points(points.as_ref()))
        }
        Constraint::LessThan { target, points } => {
            format!("LessThan {} {}", target, format_points(points.as_ref()))
        }
        Constraint::MoreThan { target, points } => {
            format!("MoreThan {} {}", target, format_points(points.as_ref()))
        }
    }
}

fn format_points(points: &std::collections::HashSet<Point>) -> String {
    let mut ordered: Vec<Point> = points.iter().copied().collect();
    ordered.sort_by_key(|point| (point.y, point.x));
    let tokens: Vec<String> = ordered
        .into_iter()
        .map(|point| format!("({}, {})", point.x, point.y))
        .collect();
    format!("{{{}}}", tokens.join(","))
}
