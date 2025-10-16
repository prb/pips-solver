use std::env;

pub mod loader;
pub mod model;
pub mod solver;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: pips-solver <file_path>");
        return;
    }

    let file_path = &args[1];

    match loader::load_game(file_path) {
        Ok(game) => match solver::solve(&game) {
            Ok(placements) => {
                for placement in placements {
                    println!("{}", placement);
                }
            }
            Err(e) => eprintln!("Error solving game: {}", e),
        },
        Err(e) => eprintln!("Error loading game: {}", e),
    }
}
