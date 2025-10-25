use pips_solver::display;
use pips_solver::generator::{GeneratorConfig, generate};
use pips_solver::model::{Game, PolyShape};
use std::env;
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let config = GeneratorConfig {
        width: args.width,
        height: args.height,
        allowed_shapes: args.shapes,
        seed: args.seed,
    };
    let puzzle = generate(config)?;

    let unsolved = display::render_unsolved(&puzzle_as_game(&puzzle));
    for line in unsolved {
        println!("{}", line);
    }
    println!();

    let piece_tokens: Vec<String> = puzzle
        .pieces
        .iter()
        .map(|piece| {
            piece
                .pips()
                .iter()
                .map(|p| p.value().to_string())
                .collect::<String>()
        })
        .collect();
    println!("pieces:{}", piece_tokens.join(","));

    println!();
    let solved = display::render_solution(&puzzle_as_game(&puzzle), &puzzle.placements);
    for line in solved {
        println!("{}", line);
    }
    Ok(())
}

fn puzzle_as_game(puzzle: &pips_solver::generator::GeneratedPuzzle) -> Game {
    Game::new(puzzle.board.clone(), puzzle.pieces.clone(), Vec::new())
}

struct CliArgs {
    width: usize,
    height: usize,
    shapes: Vec<PolyShape>,
    seed: Option<u64>,
}

fn parse_args() -> Result<CliArgs, String> {
    let mut width = 6usize;
    let mut height = 6usize;
    let mut shapes: Vec<PolyShape> = Vec::new();
    let mut seed = None;

    for arg in env::args().skip(1) {
        if let Some(value) = arg.strip_prefix("--width=") {
            width = value.parse().map_err(|_| "Invalid width".to_string())?;
        } else if let Some(value) = arg.strip_prefix("--height=") {
            height = value.parse().map_err(|_| "Invalid height".to_string())?;
        } else if let Some(value) = arg.strip_prefix("--shapes=") {
            shapes = parse_shapes(value)?;
        } else if let Some(value) = arg.strip_prefix("--seed=") {
            seed = Some(value.parse().map_err(|_| "Invalid seed".to_string())?);
        } else if arg == "--help" {
            return Err(usage());
        } else {
            return Err(usage());
        }
    }

    Ok(CliArgs {
        width,
        height,
        shapes,
        seed,
    })
}

fn parse_shapes(value: &str) -> Result<Vec<PolyShape>, String> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    let mut shapes = Vec::new();
    for token in value.split(',') {
        let trimmed = token.trim();
        let shape = match trimmed {
            "2" => PolyShape::Domino,
            "3" => PolyShape::I3,
            "4" => PolyShape::I4,
            "5" => PolyShape::I5,
            other => return Err(format!("Unknown shape length '{}'.", other)),
        };
        if !shapes.contains(&shape) {
            shapes.push(shape);
        }
    }
    Ok(shapes)
}

fn usage() -> String {
    "Usage: generate-poly [--width=N] [--height=N] [--shapes=2,3,4,5] [--seed=VALUE]".to_string()
}
