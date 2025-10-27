use pips_solver::{loader, solver_v2};
use std::env;
use std::path::PathBuf;
use std::time::Instant;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err("Usage: profile-solver-v2 <game> [<game> ...]".to_string());
    }

    for path in args {
        let absolute = canonicalize(&path)?;
        println!("profiling {}", absolute.display());
        let game = loader::load_game_from_path(&absolute)?;
        let started = Instant::now();
        match solver_v2::solve(&game) {
            Ok(solution) => {
                let elapsed = started.elapsed();
                println!(
                    "  solved with {} placements in {:.3?}",
                    solution.len(),
                    elapsed
                );
            }
            Err(err) => {
                println!("  failed: {}", err);
            }
        }
        println!();
    }
    Ok(())
}

fn canonicalize(input: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(input);
    if path.exists() {
        Ok(path)
    } else {
        Err(format!("{} not found", input))
    }
}
