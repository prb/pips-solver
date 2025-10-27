use pips_solver::{display, loader, solver, solver_v2};
use std::env;
use std::process;
use std::time::Instant;

struct CliOptions {
    show_game: bool,
    show_playout: bool,
    path: String,
    solver: SolverKind,
}

enum SolverKind {
    Legacy,
    V2,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_args()?;
    let game = loader::load_game_from_path(&options.path)?;

    if options.show_game {
        let unsolved = display::render_unsolved(&game);
        if !unsolved.is_empty() {
            for line in &unsolved {
                println!("{}", line);
            }
            let piece_lines = display::render_dominoes(&game.pieces);
            if !piece_lines.is_empty() {
                println!("\nPieces:\n");
                for line in piece_lines {
                    println!("{}", line);
                }
            }
            println!();
        }
    }

    let started = Instant::now();
    let placements = match options.solver {
        SolverKind::Legacy => solver::solve(&game),
        SolverKind::V2 => solver_v2::solve(&game),
    }?;
    let elapsed = started.elapsed();

    if options.show_playout {
        println!("Playout:\n");
        for (index, placement) in placements.iter().enumerate() {
            println!("{}: {}", index + 1, placement);
        }
        println!();
    }

    println!("Found a solution in {:?}", elapsed);
    println!();
    let rendered = display::render_solution(&game, &placements);
    for line in rendered {
        println!("{}", line);
    }
    Ok(())
}

fn parse_args() -> Result<CliOptions, String> {
    let mut positional = Vec::new();
    let mut show_game = false;
    let mut show_playout = false;
    let mut solver = SolverKind::V2;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--show-game" => show_game = true,
            "--show-playout" => show_playout = true,
            other if other.starts_with("--") => {
                if let Some(value) = other.strip_prefix("--solver=") {
                    solver = parse_solver_flag(value)?;
                } else {
                    return Err(format!("Unknown flag '{}'.", other));
                }
            }
            other => positional.push(other.to_string()),
        }
    }

    if positional.len() != 1 {
        return Err(
            "Usage: solve-polypips [--show-game] [--show-playout] <path-to-game-file>".to_string(),
        );
    }

    Ok(CliOptions {
        show_game,
        show_playout,
        path: positional.remove(0),
        solver,
    })
}

fn parse_solver_flag(value: &str) -> Result<SolverKind, String> {
    match value.to_ascii_lowercase().as_str() {
        "legacy" | "v1" => Ok(SolverKind::Legacy),
        "v2" | "new" => Ok(SolverKind::V2),
        other => Err(format!(
            "Unsupported solver '{}'. Expected 'v2' or 'legacy'.",
            other
        )),
    }
}
