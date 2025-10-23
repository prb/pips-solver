use chrono::{NaiveDate, Utc};
use pips_solver::display;
use pips_solver::loader::nyt::{self, Difficulty, NytPuzzle};
use pips_solver::solver;
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
    let usage = "Usage: solve-pips <YYYY-MM-DD> <easy|medium|hard|all>";
    let date_str = args.next().ok_or_else(|| usage.to_string())?;
    let difficulty_str = args.next().ok_or_else(|| usage.to_string())?;
    if args.next().is_some() {
        return Err(usage.to_string());
    }

    let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date '{}'. Expected YYYY-MM-DD.", date_str))?;
    let today = Utc::now().date_naive();
    if date > today {
        return Err(format!(
            "Date {} is in the future (today is {}).",
            date, today
        ));
    }

    let difficulty_token = difficulty_str.to_ascii_lowercase();
    if difficulty_token == "all" {
        let puzzle = nyt::fetch_puzzle(date)?;
        solve_all(&puzzle, date)?;
    } else {
        let difficulty = parse_difficulty(&difficulty_token)?;
        let puzzle = nyt::fetch_puzzle(date)?;
        solve_single(&puzzle, date, difficulty)?;
    }
    Ok(())
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

fn solve_all(puzzle: &NytPuzzle, date: NaiveDate) -> Result<(), String> {
    for (idx, difficulty) in Difficulty::all().iter().copied().enumerate() {
        if idx > 0 {
            println!();
        }
        println!("== {} ({}) ==", date, difficulty.display_name());
        solve_and_print(puzzle, date, difficulty)?;
    }
    Ok(())
}

fn solve_single(puzzle: &NytPuzzle, date: NaiveDate, difficulty: Difficulty) -> Result<(), String> {
    solve_and_print(puzzle, date, difficulty)
}

fn solve_and_print(
    puzzle: &NytPuzzle,
    date: NaiveDate,
    difficulty: Difficulty,
) -> Result<(), String> {
    let game = puzzle.game(difficulty)?;
    println!("Solving {} {}", date, difficulty.display_name());
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
