use chrono::{NaiveDate, Utc};
use pips_solver::display;
use pips_solver::loader::nyt::{self, Difficulty, NytPuzzle};
use pips_solver::solver;
use std::env;
use std::process;
use std::time::Instant;

struct CliArgs {
    show_game: bool,
    show_playout: bool,
    date: String,
    difficulty: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;

    let date = NaiveDate::parse_from_str(&args.date, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date '{}'. Expected YYYY-MM-DD.", args.date))?;
    let today = Utc::now().date_naive();
    if date > today {
        return Err(format!(
            "Date {} is in the future (today is {}).",
            date, today
        ));
    }

    let run_opts = RunOptions {
        show_game: args.show_game,
        show_playout: args.show_playout,
    };

    if args.difficulty == "all" {
        let puzzle = nyt::fetch_puzzle(date)?;
        solve_all(&puzzle, date, &run_opts)?;
    } else {
        let difficulty = parse_difficulty(&args.difficulty)?;
        let puzzle = nyt::fetch_puzzle(date)?;
        solve_single(&puzzle, date, difficulty, &run_opts)?;
    }
    Ok(())
}

fn parse_args() -> Result<CliArgs, String> {
    let mut show_game = false;
    let mut show_playout = false;
    let mut positional = Vec::new();

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

    if positional.len() != 2 {
        return Err(
            "Usage: solve-pips [--show-game] [--show-playout] <YYYY-MM-DD> <easy|medium|hard|all>"
                .to_string(),
        );
    }

    Ok(CliArgs {
        show_game,
        show_playout,
        date: positional.remove(0),
        difficulty: positional.remove(0).to_ascii_lowercase(),
    })
}

#[derive(Clone, Copy)]
struct RunOptions {
    show_game: bool,
    show_playout: bool,
}

fn parse_difficulty(token: &str) -> Result<Difficulty, String> {
    match token {
        "easy" => Ok(Difficulty::Easy),
        "medium" => Ok(Difficulty::Medium),
        "hard" => Ok(Difficulty::Hard),
        other => Err(format!(
            "Unknown difficulty '{}'. Expected easy, medium, hard, or all.",
            other
        )),
    }
}

fn solve_all(puzzle: &NytPuzzle, date: NaiveDate, options: &RunOptions) -> Result<(), String> {
    for (idx, difficulty) in Difficulty::all().iter().copied().enumerate() {
        if idx > 0 {
            println!();
        }
        println!("== {} ({}) ==", date, difficulty.display_name());
        solve_and_print(puzzle, date, difficulty, options)?;
    }
    Ok(())
}

fn solve_single(
    puzzle: &NytPuzzle,
    date: NaiveDate,
    difficulty: Difficulty,
    options: &RunOptions,
) -> Result<(), String> {
    solve_and_print(puzzle, date, difficulty, options)
}

fn solve_and_print(
    puzzle: &NytPuzzle,
    date: NaiveDate,
    difficulty: Difficulty,
    options: &RunOptions,
) -> Result<(), String> {
    let game = puzzle.game(difficulty)?;
    println!("Solving {} {}", date, difficulty.display_name());

    if options.show_game {
        let unsolved = display::render_unsolved(&game);
        if !unsolved.is_empty() {
            println!();
            for line in &unsolved {
                println!("{}", line);
            }
            let domino_lines = display::render_dominoes(&game.pieces);
            if !domino_lines.is_empty() {
                println!("\nPieces:\n");
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
