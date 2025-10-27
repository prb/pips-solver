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
            let digits: String = piece.pips().iter().map(|p| p.value().to_string()).collect();
            format!("{}:{}", piece.shape().code(), digits)
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
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().all(|c| c.is_ascii_digit()) {
            match trimmed {
                "1" => add_shapes(&mut shapes, &[PolyShape::Mono]),
                "2" => add_shapes(&mut shapes, &[PolyShape::Domino]),
                "3" => add_shapes(&mut shapes, &[PolyShape::TriI, PolyShape::TriL]),
                "4" => add_shapes(
                    &mut shapes,
                    &[
                        PolyShape::TetI,
                        PolyShape::TetLPlus,
                        PolyShape::TetLMinus,
                        PolyShape::TetO,
                        PolyShape::TetSPlus,
                        PolyShape::TetSMinus,
                        PolyShape::TetT,
                    ],
                ),
                "5" => add_shapes(
                    &mut shapes,
                    &[
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
                    ],
                ),
                other => return Err(format!("Unknown shape length '{}'.", other)),
            }
        } else {
            let normalized = trimmed.trim_end_matches(':');
            let shape = PolyShape::from_code(normalized)
                .ok_or_else(|| format!("Unknown shape code '{}'.", trimmed))?;
            add_shapes(&mut shapes, &[shape]);
        }
    }
    Ok(shapes)
}

fn add_shapes(acc: &mut Vec<PolyShape>, shapes: &[PolyShape]) {
    for shape in shapes {
        if !acc.contains(shape) {
            acc.push(*shape);
        }
    }
}

fn usage() -> String {
    "Usage: generate-poly [--width=N] [--height=N] [--shapes=2,3,4,5] [--seed=VALUE]".to_string()
}
