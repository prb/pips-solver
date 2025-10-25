use pips_solver::{display, loader, solver};
use std::env;
use std::process;
use std::time::Instant;

struct Options {
    show_game: bool,
    show_playout: bool,
    path: String,
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
            let domino_lines = display::render_dominoes(&game.pieces);
            if !domino_lines.is_empty() {
                println!("\nDominoes:\n");
                for line in domino_lines {
                    println!("{}", line);
                }
            }
            println!();
        }
    }

    let started = Instant::now();
    let placements = solver::solve(&game)?;
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

fn parse_args() -> Result<Options, String> {
    let mut show_game = false;
    let mut show_playout = false;
    let mut positional: Vec<String> = Vec::new();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--show-game" => show_game = true,
            "--show-playout" => show_playout = true,
            other if other.starts_with("--") => {
                return Err(format!("Unknown flag '{}'.", other));
            }
            other => positional.push(other.to_string()),
        }
    }

    if positional.len() != 1 {
        return Err(
            "Usage: pips-solver [--show-game] [--show-playout] <path-to-game-file>".to_string(),
        );
    }

    Ok(Options {
        show_game,
        show_playout,
        path: positional.remove(0),
    })
}
