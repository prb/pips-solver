mod loader;
mod model;
mod solver;

use std::env;
use std::process;

fn render_board(game: &model::Game, placements: &[model::Placement]) -> Vec<String> {
    let points = game.board.points();
    if points.is_empty() {
        return Vec::new();
    }

    let min_x = points.iter().map(|p| p.x).min().unwrap();
    let max_x = points.iter().map(|p| p.x).max().unwrap();
    let min_y = points.iter().map(|p| p.y).min().unwrap();
    let max_y = points.iter().map(|p| p.y).max().unwrap();

    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;

    let mut grid = vec![vec![' '; width]; height];

    for point in points {
        let row = (point.y - min_y) as usize;
        let col = (point.x - min_x) as usize;
        grid[row][col] = '#';
    }

    for placement in placements {
        for assignment in placement.assignments() {
            let point = assignment.point;
            if point.x < min_x || point.x > max_x || point.y < min_y || point.y > max_y {
                continue;
            }
            let row = (point.y - min_y) as usize;
            let col = (point.x - min_x) as usize;
            if let Some(ch) = char::from_digit(assignment.pips.value() as u32, 10) {
                grid[row][col] = ch;
            }
        }
    }

    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect()
}

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
    let rendered = render_board(&game, &placements);
    if !rendered.is_empty() {
        println!("\nBoard:");
        for line in rendered {
            println!("{}", line);
        }
    }
    Ok(())
}
