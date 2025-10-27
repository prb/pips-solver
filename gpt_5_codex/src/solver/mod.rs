use crate::model::{
    Board, Constraint, Game, Piece, Placement, Point, reduce_constraints, remove_one,
};
use std::collections::HashMap;

pub fn solve(game: &Game) -> Result<Vec<Placement>, String> {
    let pieces = game.pieces.clone();
    let catalog = PlacementCatalog::new(&game.board, &pieces, &game.constraints);
    let mut cover = ExactCover::new(&catalog);
    let mut solution_rows = Vec::new();
    let mut best: Option<Vec<Placement>> = None;

    cover.search(&mut solution_rows, &mut |rows| {
        if let (Some(placements), _) = assign_pips(game, &catalog, rows, true) {
            best = Some(placements);
            true
        } else {
            false
        }
    });

    best.ok_or_else(|| "No valid placements.".to_string())
}

pub fn count_solutions(game: &Game) -> Result<usize, String> {
    let pieces = game.pieces.clone();
    let catalog = PlacementCatalog::new(&game.board, &pieces, &game.constraints);
    let mut cover = ExactCover::new(&catalog);
    let mut solution_rows = Vec::new();
    let mut total = 0usize;

    cover.search(&mut solution_rows, &mut |rows| {
        let (_, count) = assign_pips(game, &catalog, rows, false);
        total += count;
        false
    });

    Ok(total)
}

fn assign_pips(
    game: &Game,
    catalog: &PlacementCatalog,
    rows: &[usize],
    stop_at_first: bool,
) -> (Option<Vec<Placement>>, usize) {
    let mut entries: Vec<&PlacementEntry> = rows.iter().map(|&idx| &catalog.entries[idx]).collect();
    entries.sort_by(|a, b| {
        b.constraint_score
            .cmp(&a.constraint_score)
            .then_with(|| b.cell_columns.len().cmp(&a.cell_columns.len()))
            .then_with(|| a.piece_shape_order.cmp(&b.piece_shape_order))
    });

    let mut placements = Vec::with_capacity(entries.len());
    let mut best: Option<Vec<Placement>> = None;
    let mut count = 0usize;
    assign_pips_recursive(
        game,
        &entries,
        0,
        &mut placements,
        &mut best,
        &mut count,
        stop_at_first,
    );
    (best, count)
}

fn assign_pips_recursive(
    state: &Game,
    entries: &[&PlacementEntry],
    index: usize,
    placements: &mut Vec<Placement>,
    best: &mut Option<Vec<Placement>>,
    count: &mut usize,
    stop_at_first: bool,
) -> bool {
    if index == entries.len() {
        *count += 1;
        if best.is_none() {
            *best = Some(placements.clone());
        }
        return stop_at_first;
    }

    let entry = entries[index];
    for pip_order in entry.piece.pip_permutations() {
        let placement = Placement::new(
            entry.piece.clone(),
            entry.anchor,
            entry.orientation_index,
            pip_order.clone(),
        );
        match play(state, &placement) {
            Ok(next_state) => {
                placements.push(placement);
                if assign_pips_recursive(
                    &next_state,
                    entries,
                    index + 1,
                    placements,
                    best,
                    count,
                    stop_at_first,
                ) {
                    return true;
                }
                placements.pop();
            }
            Err(_err) => {
                #[cfg(test)]
                eprintln!("placement failure");
                continue;
            }
        }
    }

    false
}

fn play(game: &Game, placement: &Placement) -> Result<Game, String> {
    let placement_points = placement.points();
    let board_result = game.board.remove_points(&placement_points);
    let pieces_result = remove_one(game.pieces.clone(), &placement.piece);
    let constraints_result = reduce_constraints(&game.constraints, placement);

    match (board_result, pieces_result, constraints_result) {
        (Ok(board), Ok(pieces), Ok(constraints)) => Ok(Game::new(board, pieces, constraints)),
        _ => Err("Unwinnable game.".to_string()),
    }
}

struct PlacementEntry {
    piece_index: usize,
    piece: Piece,
    piece_shape_order: usize,
    anchor: Point,
    orientation_index: usize,
    cell_columns: Vec<usize>,
    constraint_score: usize,
}

struct PlacementCatalog {
    entries: Vec<PlacementEntry>,
    board_cell_count: usize,
    piece_count: usize,
}

impl PlacementCatalog {
    fn new(board: &Board, pieces: &[Piece], constraints: &[Constraint]) -> Self {
        let mut index_map = HashMap::new();
        for (idx, point) in board.iter().enumerate() {
            index_map.insert(point, idx);
        }

        if index_map.is_empty() {
            return Self {
                entries: Vec::new(),
                board_cell_count: 0,
                piece_count: pieces.len(),
            };
        }

        let mut entries = Vec::new();

        for (piece_index, piece) in pieces.iter().enumerate() {
            let piece_shape_order = piece.shape().cell_count();
            for (orientation_index, offsets) in piece.orientations().iter().enumerate() {
                for anchor in board.iter() {
                    let mut cell_columns = Vec::with_capacity(offsets.len());
                    let mut valid = true;
                    for &(dx, dy) in offsets {
                        let x = anchor.x as i32 + dx;
                        let y = anchor.y as i32 + dy;
                        if x < 0 || y < 0 {
                            valid = false;
                            break;
                        }
                        let point = Point::new(x as u32, y as u32);
                        if let Some(&index) = index_map.get(&point) {
                            cell_columns.push(index);
                        } else {
                            valid = false;
                            break;
                        }
                    }

                    if !valid {
                        continue;
                    }

                    cell_columns.sort_unstable();

                    let mut constraint_score = 0usize;
                    for constraint in constraints {
                        if constraint.points().iter().any(|p| {
                            index_map
                                .get(p)
                                .map(|idx| cell_columns.contains(idx))
                                .unwrap_or(false)
                        }) {
                            constraint_score += 1;
                        }
                    }

                    entries.push(PlacementEntry {
                        piece_index,
                        piece: piece.clone(),
                        piece_shape_order,
                        anchor,
                        orientation_index,
                        cell_columns: cell_columns.clone(),
                        constraint_score,
                    });
                }
            }
        }

        Self {
            entries,
            board_cell_count: index_map.len(),
            piece_count: pieces.len(),
        }
    }
}

struct ExactCover {
    column_rows: Vec<Vec<usize>>,
    row_columns: Vec<Vec<usize>>,
    active_columns: Vec<bool>,
    active_rows: Vec<bool>,
    column_size: Vec<usize>,
}

impl ExactCover {
    fn new(catalog: &PlacementCatalog) -> Self {
        let row_count = catalog.entries.len();
        let column_count = catalog.board_cell_count + catalog.piece_count;

        let mut row_columns = Vec::with_capacity(row_count);
        let mut column_rows: Vec<Vec<usize>> = vec![Vec::new(); column_count];

        for (row_index, entry) in catalog.entries.iter().enumerate() {
            let mut columns = entry.cell_columns.clone();
            columns.push(catalog.board_cell_count + entry.piece_index);
            columns.sort_unstable();
            for &column in &columns {
                column_rows[column].push(row_index);
            }
            row_columns.push(columns);
        }

        let column_size = column_rows.iter().map(|rows| rows.len()).collect();
        let active_columns = vec![true; column_count];
        let active_rows = vec![true; row_count];

        Self {
            column_rows,
            row_columns,
            active_columns,
            active_rows,
            column_size,
        }
    }

    fn search<F>(&mut self, solution: &mut Vec<usize>, callback: &mut F) -> bool
    where
        F: FnMut(&[usize]) -> bool,
    {
        let column = match self.select_column() {
            Some(index) => index,
            None => {
                return callback(solution);
            }
        };

        if self.column_size[column] == 0 {
            return false;
        }

        let state = self.cover_column(column);
        let rows = self.column_rows[column].clone();
        for row in rows {
            solution.push(row);
            let mut row_states = Vec::new();
            let row_columns = self.row_columns[row].clone();
            for col in row_columns {
                if col == column {
                    continue;
                }
                if self.active_columns[col] {
                    row_states.push(self.cover_column(col));
                }
            }

            if self.search(solution, callback) {
                return true;
            }

            for state in row_states.into_iter().rev() {
                self.uncover_column(state);
            }
            solution.pop();
        }

        self.uncover_column(state);
        false
    }

    fn select_column(&self) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_size = usize::MAX;
        for (index, active) in self.active_columns.iter().enumerate() {
            if !*active {
                continue;
            }
            let size = self.column_size[index];
            if size == 0 {
                return Some(index);
            }
            if size < best_size {
                best_size = size;
                best = Some(index);
                if size == 1 {
                    break;
                }
            }
        }
        best
    }

    fn cover_column(&mut self, column: usize) -> CoverState {
        let prev_size = self.column_size[column];
        self.active_columns[column] = false;
        self.column_size[column] = 0;

        let mut rows_removed = Vec::new();
        for &row in &self.column_rows[column] {
            if !self.active_rows[row] {
                continue;
            }
            self.active_rows[row] = false;
            let mut affected = Vec::new();
            for &col in &self.row_columns[row] {
                if self.active_columns[col] {
                    self.column_size[col] -= 1;
                    affected.push(col);
                }
            }
            rows_removed.push(RowRemoval { row, affected });
        }

        CoverState {
            column,
            column_prev_size: prev_size,
            rows_removed,
        }
    }

    fn uncover_column(&mut self, state: CoverState) {
        for removal in state.rows_removed.into_iter().rev() {
            self.active_rows[removal.row] = true;
            for col in removal.affected {
                if self.active_columns[col] {
                    self.column_size[col] += 1;
                }
            }
        }
        self.column_size[state.column] = state.column_prev_size;
        self.active_columns[state.column] = true;
    }
}

struct CoverState {
    column: usize,
    column_prev_size: usize,
    rows_removed: Vec<RowRemoval>,
}

struct RowRemoval {
    row: usize,
    affected: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::{count_solutions, solve};
    use crate::model::{Board, Constraint, Game, Piece, Pips, Point, PolyShape};
    use std::collections::HashSet;
    use std::sync::Arc;

    #[test]
    fn solves_single_piece_board() {
        let mut points = HashSet::new();
        points.insert(Point::new(0, 0));
        points.insert(Point::new(1, 0));
        let board = Board::new(points);
        let pieces = vec![Piece::domino(Pips::new(1).unwrap(), Pips::new(1).unwrap())];
        let game = Game::new(board, pieces, vec![]);
        game.validate().unwrap();
        let solution = solve(&game).expect("solution should exist");
        assert_eq!(solution.len(), 1);
    }

    #[test]
    fn counts_single_solution() {
        let mut points = HashSet::new();
        points.insert(Point::new(0, 0));
        points.insert(Point::new(1, 0));
        let board = Board::new(points);
        let pieces = vec![Piece::domino(Pips::new(1).unwrap(), Pips::new(1).unwrap())];
        let game = Game::new(board, pieces, vec![]);
        game.validate().unwrap();
        let total = count_solutions(&game).expect("count should succeed");
        assert_eq!(total, 1);
    }

    #[test]
    fn counts_zero_for_unsolved_game() {
        let mut points = HashSet::new();
        points.insert(Point::new(0, 0));
        points.insert(Point::new(1, 0));
        let board = Board::new(points.clone());

        let pieces = vec![Piece::domino(Pips::new(1).unwrap(), Pips::new(2).unwrap())];
        let mut constraint_points = HashSet::new();
        constraint_points.insert(Point::new(0, 0));
        constraint_points.insert(Point::new(1, 0));
        let constraints = vec![Constraint::Exactly {
            target: 10,
            points: Arc::new(constraint_points),
        }];
        let game = Game::new(board, pieces, constraints);
        game.validate().unwrap();
        let total = count_solutions(&game).expect("count should succeed");
        assert_eq!(total, 0);
    }

    #[test]
    fn solves_straight_tri_line() {
        let mut points = HashSet::new();
        points.insert(Point::new(0, 0));
        points.insert(Point::new(1, 0));
        points.insert(Point::new(2, 0));
        let board = Board::new(points);
        let piece = Piece::new(
            PolyShape::TriI,
            vec![
                Pips::new(1).unwrap(),
                Pips::new(1).unwrap(),
                Pips::new(1).unwrap(),
            ],
        )
        .unwrap();
        let game = Game::new(board, vec![piece], vec![]);
        game.validate().unwrap();
        let solution = solve(&game).expect("solution should exist");
        assert_eq!(solution.len(), 1);
    }
}
