mod polyomino;
mod board;
mod solver;
mod renderer;

use board::Board;
use solver::solve;
use renderer::render_solution;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "proof-8x8-2x2")]
#[command(about = "Generate pentomino tilings for 8x8 boards with 2x2 holes", long_about = None)]
struct Args {
    /// Use colored output for pentomino labels
    #[arg(long)]
    color: bool,

    /// Arrange output grids in the same layout as the hole positions
    #[arg(long)]
    pretty_layout: bool,
}

fn main() {
    let args = Args::parse();

    // The 10 unique positions for the 2x2 hole (upper-left corner)
    let hole_positions = [
        (0, 0), (1, 0), (2, 0), (3, 0),
        (1, 1), (2, 1), (3, 1),
        (2, 2), (3, 2),
        (3, 3),
    ];

    // Generate all solutions
    let mut solutions = Vec::new();
    for &(hole_x, hole_y) in &hole_positions {
        let board = Board::new(8, 8, hole_x, hole_y);
        let solution = solve(&board);
        solutions.push((hole_x, hole_y, board, solution));
    }

    if args.pretty_layout {
        render_pretty_layout(&solutions, args.color);
    } else {
        render_sequential(&solutions, args.color);
    }
}

fn render_sequential(solutions: &[(usize, usize, Board, Option<Vec<polyomino::Placement>>)], use_color: bool) {
    for (idx, &(hole_x, hole_y, ref board, ref solution)) in solutions.iter().enumerate() {
        println!("Case {}: Hole at ({}, {})", idx + 1, hole_x, hole_y);
        println!();

        match solution {
            Some(sol) => {
                let rendered = render_solution(board, sol, use_color);
                for line in rendered {
                    println!("{}", line);
                }
            }
            None => {
                println!("No solution found!");
            }
        }

        println!();
    }
}

fn render_pretty_layout(solutions: &[(usize, usize, Board, Option<Vec<polyomino::Placement>>)], use_color: bool) {
    // Organize solutions by their position in the grid
    // Row 0: positions (0,0), (1,0), (2,0), (3,0) - indices 0, 1, 2, 3
    // Row 1: positions (1,1), (2,1), (3,1) - indices 4, 5, 6
    // Row 2: positions (2,2), (3,2) - indices 7, 8
    // Row 3: position (3,3) - index 9

    let rows = vec![
        vec![0, 1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8],
        vec![9],
    ];

    // Calculate grid width and spacing
    let grid_spacing = 4;
    let mut grid_width = 0;

    for (row_idx, row) in rows.iter().enumerate() {
        // Render all grids in this row (without labels first)
        let grids: Vec<Vec<String>> = row.iter().map(|&idx| {
            let (_hole_x, _hole_y, ref board, ref solution) = solutions[idx];
            match solution {
                Some(sol) => {
                    render_solution(board, sol, use_color)
                }
                None => {
                    vec!["No solution found!".to_string()]
                }
            }
        }).collect();

        let labels: Vec<String> = row.iter().map(|&idx| {
            let (hole_x, hole_y, _, _) = solutions[idx];
            format!("Hole at ({}, {})", hole_x, hole_y)
        }).collect();

        // Find max height and grid width (from first row)
        let max_height = grids.iter().map(|g| g.len()).max().unwrap_or(0);

        if row_idx == 0 && !grids.is_empty() {
            // Calculate grid width from first grid, first actual grid line
            if let Some(first_line) = grids[0].get(0) {
                grid_width = strip_ansi_codes(first_line).chars().count();
            }
        }

        // Calculate left padding for right-justification
        // Missing grids count from the full row of 4
        let missing_grids = 4 - row.len();
        let left_padding = if missing_grids > 0 {
            missing_grids * grid_width + missing_grids * grid_spacing
        } else {
            0
        };

        // Print labels with right-justification
        print!("{}", " ".repeat(left_padding));
        for (grid_idx, label) in labels.iter().enumerate() {
            if grid_idx > 0 {
                print!("{}", " ".repeat(grid_spacing));
            }
            // Print label and pad to grid width
            let label_len = label.chars().count();
            print!("{}", label);
            if label_len < grid_width {
                print!("{}", " ".repeat(grid_width - label_len));
            }
        }
        println!();

        // Print blank line
        println!();

        // Print grids side by side with proper right-justification
        for line_idx in 0..max_height {
            // Add left padding for right-justification
            print!("{}", " ".repeat(left_padding));

            for (grid_idx, grid) in grids.iter().enumerate() {
                if grid_idx > 0 {
                    print!("{}", " ".repeat(grid_spacing)); // Spacing between grids
                }

                if line_idx < grid.len() {
                    print!("{}", grid[line_idx]);
                } else {
                    // Pad with spaces if this grid is shorter
                    print!("{}", " ".repeat(grid_width));
                }
            }
            println!();
        }

        // Add spacing between rows
        if row_idx < rows.len() - 1 {
            println!();
        }
    }
}

fn strip_ansi_codes(s: &str) -> String {
    // Simple ANSI code stripper for measuring actual string width
    let mut result = String::new();
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape && ch == 'm' {
            in_escape = false;
        } else if !in_escape {
            result.push(ch);
        }
    }
    result
}
