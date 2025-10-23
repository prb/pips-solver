use pips_solver::{display, loader, solver};
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
    let path = args
        .next()
        .ok_or_else(|| "Usage: pips-solver <path-to-game-file>".to_string())?;
    if args.next().is_some() {
        return Err("Usage: pips-solver <path-to-game-file>".to_string());
    }

    let game = loader::load_game_from_path(&path)?;
    let placements = solver::solve(&game)?;
    for (index, placement) in placements.iter().enumerate() {
        println!("{}: {}", index + 1, placement);
    }
    let rendered = display::render_board(&game, &placements);
    if !rendered.is_empty() {
        println!("\nBoard:");
        for line in rendered {
            println!("{}", line);
        }
    }
    Ok(())
}
