// Pips Solver - NY Times Pips puzzle solver
// A learning project exploring Rust programming

mod data_model;
mod loader;
mod solver;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <game_file>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    // Load the game from file
    let game = match loader::load_game(file_path) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Error loading game: {}", e);
            process::exit(1);
        }
    };

    // Solve the game
    let solution = match solver::solve(game) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error solving game: {}", e);
            process::exit(1);
        }
    };

    // Output the solution
    println!("Solution found with {} placements:", solution.len());
    for (i, placement) in solution.iter().enumerate() {
        println!("{}. {:?}", i + 1, placement);
    }
}
