use crate::board::Board;
use crate::polyomino::{Placement, Pentomino};
use std::collections::HashMap;
use colored::*;

const CELL_WIDTH: usize = 3;

fn pentomino_color(pentomino: Pentomino) -> Color {
    match pentomino {
        Pentomino::F => Color::Red,
        Pentomino::I => Color::Green,
        Pentomino::L => Color::Yellow,
        Pentomino::N => Color::Blue,
        Pentomino::P => Color::Magenta,
        Pentomino::T => Color::Cyan,
        Pentomino::U => Color::BrightRed,
        Pentomino::V => Color::BrightGreen,
        Pentomino::W => Color::BrightYellow,
        Pentomino::X => Color::BrightBlue,
        Pentomino::Y => Color::BrightMagenta,
        Pentomino::Z => Color::BrightCyan,
    }
}

pub fn render_solution(board: &Board, placements: &[Placement], use_color: bool) -> Vec<String> {
    // Create a map from cell to (pentomino, label)
    let mut cell_map: HashMap<(usize, usize), (Pentomino, char)> = HashMap::new();

    for placement in placements {
        for &cell in &placement.cells {
            cell_map.insert(cell, (placement.pentomino, placement.pentomino.label()));
        }
    }

    let width = board.width;
    let height = board.height;

    // Create the drawing grid
    let draw_rows = height * 2 + 1;
    let draw_cols = width * (CELL_WIDTH + 1) + 1;
    let mut grid = vec![vec![' '; draw_cols]; draw_rows];

    // Track pentomino colors for each position in the grid
    let mut color_grid: Vec<Vec<Option<Color>>> = vec![vec![None; draw_cols]; draw_rows];

    // Track which edges need borders
    let mut nodes = vec![vec![NodeEdges::default(); width + 1]; height + 1];

    // Fill in cell contents and determine borders
    for y in 0..height {
        for x in 0..width {
            let base_row = y * 2;
            let base_col = x * (CELL_WIDTH + 1);

            let current_entry = cell_map.get(&(x, y));
            let current_pentomino = current_entry.map(|(p, _)| p);

            // Check borders with neighbors (based on pentomino type, not label)
            let north_different = y == 0 || cell_map.get(&(x, y.wrapping_sub(1))).map(|(p, _)| p) != current_pentomino;
            let south_different = y + 1 >= height || cell_map.get(&(x, y + 1)).map(|(p, _)| p) != current_pentomino;
            let west_different = x == 0 || cell_map.get(&(x.wrapping_sub(1), y)).map(|(p, _)| p) != current_pentomino;
            let east_different = x + 1 >= width || cell_map.get(&(x + 1, y)).map(|(p, _)| p) != current_pentomino;

            // Draw horizontal borders
            if north_different {
                for offset in 1..=CELL_WIDTH {
                    grid[base_row][base_col + offset] = '─';
                }
                nodes[y][x].east = true;
                nodes[y][x + 1].west = true;
            }

            if south_different {
                for offset in 1..=CELL_WIDTH {
                    grid[base_row + 2][base_col + offset] = '─';
                }
                nodes[y + 1][x].east = true;
                nodes[y + 1][x + 1].west = true;
            }

            // Draw vertical borders
            if west_different {
                grid[base_row + 1][base_col] = '│';
                nodes[y][x].south = true;
                nodes[y + 1][x].north = true;
            }

            if east_different {
                grid[base_row + 1][base_col + CELL_WIDTH + 1] = '│';
                nodes[y][x + 1].south = true;
                nodes[y + 1][x + 1].north = true;
            }

            // Fill in cell label
            if let Some(&(pentomino, label)) = current_entry {
                let text = center_text(&label.to_string(), CELL_WIDTH);
                let color = pentomino_color(pentomino);
                for (i, ch) in text.chars().enumerate() {
                    let col_idx = base_col + 1 + i;
                    grid[base_row + 1][col_idx] = ch;
                    color_grid[base_row + 1][col_idx] = Some(color);
                }
            }
        }
    }

    // Draw corner nodes
    for y in 0..=height {
        for x in 0..=width {
            let edges = nodes[y][x];
            let ch = edges.to_char();
            let draw_row = y * 2;
            let draw_col = x * (CELL_WIDTH + 1);
            grid[draw_row][draw_col] = ch;
        }
    }

    // Convert grid to strings with optional colors
    grid.into_iter()
        .zip(color_grid.into_iter())
        .map(|(line, colors)| {
            let mut result = String::new();
            for (ch, color_opt) in line.into_iter().zip(colors.into_iter()) {
                let ch_str = ch.to_string();
                let colored_str = if use_color {
                    if let Some(color) = color_opt {
                        ch_str.color(color).to_string()
                    } else {
                        ch_str
                    }
                } else {
                    ch_str
                };
                result.push_str(&colored_str);
            }
            result.trim_end().to_string()
        })
        .filter(|line| !line.is_empty())
        .collect()
}

fn center_text(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if len >= width {
        text.chars().take(width).collect()
    } else {
        let padding = width - len;
        let left = padding / 2;
        let right = padding - left;
        format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
    }
}

#[derive(Copy, Clone, Default)]
struct NodeEdges {
    north: bool,
    south: bool,
    east: bool,
    west: bool,
}

impl NodeEdges {
    fn to_char(self) -> char {
        let mut bits = 0u8;
        if self.north {
            bits |= 1;
        }
        if self.south {
            bits |= 2;
        }
        if self.east {
            bits |= 4;
        }
        if self.west {
            bits |= 8;
        }
        match bits {
            0 => ' ',
            1 | 2 | 3 => '│',
            4 | 8 | 12 => '─',
            5 => '└',
            6 => '┌',
            7 => '├',
            9 => '┘',
            10 => '┐',
            11 => '┤',
            13 => '┴',
            14 => '┬',
            15 => '┼',
            _ => ' ',
        }
    }
}
