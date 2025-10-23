use pips_solver::{display, loader, solver};
use std::env;
use std::process;
use std::time::Instant;

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let path = args
        .next()
        .ok_or_else(|| "Usage: pips-solver <path-to-game-file>".to_string())?;
    if args.next().is_some() {
        return Err("Usage: pips-solver <path-to-game-file>".to_string());
    }

    let game = loader::load_game_from_path(&path)?;
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

    let started = Instant::now();
    let placements = solver::solve(&game)?;
    let elapsed = started.elapsed();

    for (index, placement) in placements.iter().enumerate() {
        println!("{}: {}", index + 1, placement);
    }

    println!("\nFound a solution in {:?}", elapsed);
    let rendered = display::render_solution(&game, &placements);
    for line in rendered {
        println!("{}", line);
    }
    Ok(())
}
