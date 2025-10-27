use crate::model::{
    Game, constraint::Constraint, piece::Piece, placement::Placement, point::Point,
};
use std::collections::{HashMap, HashSet};

const CELL_WIDTH: usize = 3;

pub fn render_unsolved(game: &Game) -> Vec<String> {
    let layout = BoardLayout::with_constraints(game);
    layout.render(|cell| cell.label.clone())
}

pub fn render_solution(game: &Game, placements: &[Placement]) -> Vec<String> {
    let layout = BoardLayout::with_dominoes(game, placements);
    let mut assignments = HashMap::new();
    for placement in placements {
        for assignment in placement.assignments() {
            assignments.insert(assignment.point, assignment.pips.value());
        }
    }
    layout.render(|cell| {
        assignments
            .get(&cell.point)
            .map(|value| value.to_string())
            .unwrap_or_default()
    })
}

pub fn render_dominoes(pieces: &[Piece]) -> Vec<String> {
    if pieces.is_empty() {
        return Vec::new();
    }
    let mut tokens: Vec<String> = pieces
        .iter()
        .map(|piece| {
            let values: Vec<String> = piece.pips().iter().map(|p| p.value().to_string()).collect();
            format!("{}:{}", piece.shape().code(), values.concat())
        })
        .collect();
    tokens.sort();

    const MAX_WIDTH: usize = 80;
    let mut lines = Vec::new();
    let mut current = String::new();

    for token in tokens {
        if current.is_empty() {
            current.push_str(&token);
            continue;
        }

        let projected_len = current.len() + 2 + token.len();
        if projected_len <= MAX_WIDTH {
            current.push_str(", ");
            current.push_str(&token);
        } else {
            lines.push(current);
            current = token;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

#[derive(Clone)]
struct CellData {
    point: Point,
    region: usize,
    label: String,
    constrained: bool,
}

struct BoardLayout {
    rows: usize,
    cols: usize,
    cells: Vec<Vec<Option<CellData>>>,
    fill_missing: bool,
    blend_unconstrained: bool,
}

impl BoardLayout {
    fn with_constraints(game: &Game) -> Self {
        if game.board.is_empty() {
            return Self {
                rows: 0,
                cols: 0,
                cells: Vec::new(),
                fill_missing: true,
                blend_unconstrained: true,
            };
        }

        let (min_x, max_x, min_y, max_y) = game.board.bounds().unwrap();

        let rows = (max_y - min_y + 1) as usize;
        let cols = (max_x - min_x + 1) as usize;
        let mut cells = vec![vec![None; cols]; rows];

        let mut region_map = HashMap::new();
        let mut label_points = HashMap::new();
        let mut labels = HashMap::new();
        let mut constraint_regions = HashSet::new();

        for (idx, constraint) in game.constraints.iter().enumerate() {
            let region_id = idx;
            constraint_regions.insert(region_id);
            let points_in_region = constraint.points();
            if let Some(label_point) = points_in_region
                .iter()
                .min_by_key(|point| (point.y, point.x))
                .copied()
            {
                label_points.insert(region_id, label_point);
            }
            labels.insert(region_id, label_for_constraint(constraint));
            for point in points_in_region {
                region_map.insert(*point, region_id);
            }
        }

        let mut next_region = game.constraints.len();
        for point in game.board.iter() {
            let region = *region_map.entry(point).or_insert_with(|| {
                let id = next_region;
                next_region += 1;
                id
            });
            let constrained = constraint_regions.contains(&region);
            let row = (point.y - min_y) as usize;
            let col = (point.x - min_x) as usize;
            let label_text = if let Some(label_point) = label_points.get(&region) {
                if *label_point == point {
                    labels.get(&region).cloned().unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            cells[row][col] = Some(CellData {
                point,
                region,
                label: label_text,
                constrained,
            });
        }

        label_unconstrained_regions(&mut cells);

        Self {
            rows,
            cols,
            cells,
            fill_missing: false,
            blend_unconstrained: true,
        }
    }

    fn with_dominoes(game: &Game, placements: &[Placement]) -> Self {
        if game.board.is_empty() {
            return Self {
                rows: 0,
                cols: 0,
                cells: Vec::new(),
                fill_missing: false,
                blend_unconstrained: false,
            };
        }

        let (min_x, max_x, min_y, max_y) = game.board.bounds().unwrap();

        let rows = (max_y - min_y + 1) as usize;
        let cols = (max_x - min_x + 1) as usize;
        let mut cells = vec![vec![None; cols]; rows];

        let mut region_map = HashMap::new();
        for (idx, placement) in placements.iter().enumerate() {
            for assignment in placement.assignments() {
                region_map.insert(assignment.point, idx);
            }
        }

        let mut next_region = placements.len();
        for point in game.board.iter() {
            let region = *region_map.entry(point).or_insert_with(|| {
                let id = next_region;
                next_region += 1;
                id
            });
            let row = (point.y - min_y) as usize;
            let col = (point.x - min_x) as usize;

            cells[row][col] = Some(CellData {
                point,
                region,
                label: String::new(),
                constrained: false,
            });
        }

        Self {
            rows,
            cols,
            cells,
            fill_missing: false,
            blend_unconstrained: false,
        }
    }

    fn render<F>(&self, mut text_fn: F) -> Vec<String>
    where
        F: FnMut(&CellData) -> String,
    {
        if self.rows == 0 || self.cols == 0 {
            return Vec::new();
        }

        let draw_rows = self.rows * 2 + 1;
        let draw_cols = self.cols * (CELL_WIDTH + 1) + 1;
        let mut grid = vec![vec![' '; draw_cols]; draw_rows];
        let mut nodes = vec![vec![NodeEdges::default(); self.cols + 1]; self.rows + 1];

        for row in 0..self.rows {
            for col in 0..self.cols {
                let Some(cell) = self.cells[row][col].as_ref() else {
                    if self.fill_missing {
                        Self::fill_missing_cell(&mut grid, row, col);
                    }
                    continue;
                };
                let base_row = row * 2;
                let base_col = col * (CELL_WIDTH + 1);

                let north_border = self.border_between(row as isize, col as isize, -1, 0, cell);
                let south_border = self.border_between(row as isize, col as isize, 1, 0, cell);
                let west_border = self.border_between(row as isize, col as isize, 0, -1, cell);
                let east_border = self.border_between(row as isize, col as isize, 0, 1, cell);

                if north_border {
                    for offset in 1..=CELL_WIDTH {
                        grid[base_row][base_col + offset] = '─';
                    }
                    nodes[row][col].east = true;
                    nodes[row][col + 1].west = true;
                }

                if south_border {
                    for offset in 1..=CELL_WIDTH {
                        grid[base_row + 2][base_col + offset] = '─';
                    }
                    nodes[row + 1][col].east = true;
                    nodes[row + 1][col + 1].west = true;
                }

                if west_border {
                    grid[base_row + 1][base_col] = '│';
                    nodes[row][col].south = true;
                    nodes[row + 1][col].north = true;
                }

                if east_border {
                    grid[base_row + 1][base_col + CELL_WIDTH + 1] = '│';
                    nodes[row][col + 1].south = true;
                    nodes[row + 1][col + 1].north = true;
                }

                let text = sanitize_text(&text_fn(cell));
                let formatted = center_text(&text, CELL_WIDTH);
                for (i, ch) in formatted.chars().enumerate() {
                    grid[base_row + 1][base_col + 1 + i] = ch;
                }
            }
        }

        for row in 0..=self.rows {
            for col in 0..=self.cols {
                let edges = nodes[row][col];
                let ch = edges.to_char();
                let draw_row = row * 2;
                let draw_col = col * (CELL_WIDTH + 1);
                grid[draw_row][draw_col] = ch;
            }
        }

        grid.into_iter()
            .map(|mut line| {
                while matches!(line.last(), Some(' ')) {
                    line.pop();
                }
                line.into_iter().collect::<String>()
            })
            .filter(|line| !line.is_empty())
            .collect()
    }

    fn border_between(
        &self,
        row: isize,
        col: isize,
        delta_row: isize,
        delta_col: isize,
        cell: &CellData,
    ) -> bool {
        let neighbor_row = row + delta_row;
        let neighbor_col = col + delta_col;
        if neighbor_row < 0
            || neighbor_row >= self.rows as isize
            || neighbor_col < 0
            || neighbor_col >= self.cols as isize
        {
            return true;
        }
        match &self.cells[neighbor_row as usize][neighbor_col as usize] {
            Some(neighbor) if neighbor.region == cell.region => false,
            Some(neighbor)
                if self.blend_unconstrained && !cell.constrained && !neighbor.constrained =>
            {
                false
            }
            None => true,
            _ => true,
        }
    }

    fn fill_missing_cell(grid: &mut [Vec<char>], row: usize, col: usize) {
        let _ = (grid, row, col);
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

fn sanitize_text(text: &str) -> String {
    text.chars().take(CELL_WIDTH).collect()
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

fn label_for_constraint(constraint: &Constraint) -> String {
    match constraint {
        Constraint::AllSame { expected: None, .. } => "=".to_string(),
        Constraint::AllSame {
            expected: Some(pips),
            ..
        } => format!("={}", pips.value()),
        Constraint::AllDifferent { .. } => "≠".to_string(),
        Constraint::Exactly { target, .. } => target.to_string(),
        Constraint::LessThan { target, .. } => format!("<{}", target),
        Constraint::MoreThan { target, .. } => format!(">{}", target),
    }
}

fn label_unconstrained_regions(cells: &mut [Vec<Option<CellData>>]) {
    if cells.is_empty() || cells[0].is_empty() {
        return;
    }
    let rows = cells.len();
    let cols = cells[0].len();
    let mut visited = vec![vec![false; cols]; rows];

    for row in 0..rows {
        for col in 0..cols {
            if visited[row][col] {
                continue;
            }
            let Some(cell) = cells[row][col].as_ref() else {
                continue;
            };
            if cell.constrained {
                visited[row][col] = true;
                continue;
            }

            let mut stack = vec![(row, col)];
            let mut best = (row, col, cell.point);

            while let Some((r, c)) = stack.pop() {
                if visited[r][c] {
                    continue;
                }
                visited[r][c] = true;

                let Some(current) = cells[r][c].as_ref() else {
                    continue;
                };
                if current.constrained {
                    continue;
                }
                if current.point.y < best.2.y
                    || (current.point.y == best.2.y && current.point.x < best.2.x)
                {
                    best = (r, c, current.point);
                }

                let neighbors = [
                    (r.wrapping_sub(1), c, r > 0),
                    (r + 1, c, r + 1 < rows),
                    (r, c.wrapping_sub(1), c > 0),
                    (r, c + 1, c + 1 < cols),
                ];

                for &(nr, nc, valid) in &neighbors {
                    if !valid {
                        continue;
                    }
                    if visited[nr][nc] {
                        continue;
                    }
                    match cells[nr][nc].as_ref() {
                        Some(neighbor) if !neighbor.constrained => stack.push((nr, nc)),
                        _ => continue,
                    }
                }
            }

            if let Some(cell) = cells[best.0][best.1].as_mut() {
                if cell.label.is_empty() {
                    cell.label = "∅".to_string();
                }
            }
        }
    }
}
