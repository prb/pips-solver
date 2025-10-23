use chrono::{NaiveDate, Utc};
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
    let usage = "Usage: count-solutions <YYYY-MM-DD> <easy|medium|hard|all>";
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

    let token = difficulty_str.to_ascii_lowercase();
    let puzzle = nyt::fetch_puzzle(date)?;
    if token == "all" {
        count_all(&puzzle, date)?;
    } else {
        let difficulty = parse_difficulty(&token)?;
        count_single(&puzzle, date, difficulty)?;
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

fn count_all(puzzle: &NytPuzzle, date: NaiveDate) -> Result<(), String> {
    for (idx, difficulty) in Difficulty::all().iter().copied().enumerate() {
        if idx > 0 {
            println!();
        }
        println!("== {} ({}) ==", date, difficulty.display_name());
        count_for_difficulty(puzzle, date, difficulty)?;
    }
    Ok(())
}

fn count_single(puzzle: &NytPuzzle, date: NaiveDate, difficulty: Difficulty) -> Result<(), String> {
    count_for_difficulty(puzzle, date, difficulty)
}

fn count_for_difficulty(
    puzzle: &NytPuzzle,
    date: NaiveDate,
    difficulty: Difficulty,
) -> Result<(), String> {
    let game = puzzle.game(difficulty)?;
    println!(
        "Counting solutions for {} {}",
        date,
        difficulty.display_name()
    );
    let total = solver::count_solutions(&game)?;
    println!("Total solutions: {}", total);
    Ok(())
}
