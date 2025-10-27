use crate::model::{Board, Constraint, Game, Piece, Placement, Point, reduce_constraints};
use std::collections::{HashMap, HashSet};
use std::mem;

pub fn solve(game: &Game) -> Result<Vec<Placement>, String> {
    let catalog = PlacementCatalog::from_game(game)?;
    let mut remaining = vec![true; catalog.board_points.len()];
    let mut used_pieces = vec![false; catalog.piece_count];
    let mut constraints = game.constraints.clone();
    let mut placements: Vec<Placement> = Vec::with_capacity(game.pieces.len());
    let mut cells_remaining = catalog.board_points.len();

    if search(
        game,
        &catalog,
        &mut remaining,
        &mut used_pieces,
        &mut constraints,
        &mut placements,
        &mut cells_remaining,
    ) {
        validate_solution(game, &placements)?;
        Ok(placements)
    } else {
        Err("No tiling found.".to_string())
    }
}

#[derive(Debug, Clone)]
struct PlacementRow {
    piece_index: usize,
    orientation_index: usize,
    anchor: Point,
    cell_indices: Vec<usize>,
}

struct PlacementCatalog {
    entries: Vec<PlacementRow>,
    cell_to_entries: Vec<Vec<usize>>,
    board_points: Vec<Point>,
    piece_count: usize,
}

impl PlacementCatalog {
    fn from_game(game: &Game) -> Result<Self, String> {
        let (index_map, board_points) = board_index_map(&game.board);
        if board_points.is_empty() {
            return Err("Board has no cells.".to_string());
        }

        let mut entries = Vec::new();
        for (piece_index, piece) in game.pieces.iter().enumerate() {
            entries.extend(enumerate_piece_rows(
                piece_index,
                piece,
                &game.board,
                &index_map,
            ));
        }

        let mut cell_to_entries = vec![Vec::new(); board_points.len()];
        for (entry_index, entry) in entries.iter().enumerate() {
            for &cell in &entry.cell_indices {
                cell_to_entries[cell].push(entry_index);
            }
        }

        Ok(Self {
            entries,
            cell_to_entries,
            board_points,
            piece_count: game.pieces.len(),
        })
    }
}

fn board_index_map(board: &Board) -> (HashMap<Point, usize>, Vec<Point>) {
    let mut map = HashMap::new();
    let mut points = Vec::new();
    for (idx, point) in board.iter().enumerate() {
        map.insert(point, idx);
        points.push(point);
    }
    (map, points)
}

fn enumerate_piece_rows(
    piece_index: usize,
    piece: &Piece,
    board: &Board,
    index_map: &HashMap<Point, usize>,
) -> Vec<PlacementRow> {
    let mut rows = Vec::new();
    for (orientation_index, offsets) in piece.orientations().iter().enumerate() {
        for anchor in board.iter() {
            let mut cell_indices = Vec::with_capacity(offsets.len());
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
                    cell_indices.push(index);
                } else {
                    valid = false;
                    break;
                }
            }
            if !valid {
                continue;
            }
            cell_indices.sort_unstable();
            rows.push(PlacementRow {
                piece_index,
                orientation_index,
                anchor,
                cell_indices,
            });
        }
    }
    rows
}

fn search(
    game: &Game,
    catalog: &PlacementCatalog,
    remaining: &mut [bool],
    used_pieces: &mut [bool],
    constraints: &mut Vec<Constraint>,
    placements: &mut Vec<Placement>,
    cells_remaining: &mut usize,
) -> bool {
    if *cells_remaining == 0 {
        return constraints.is_empty();
    }

    let pivot = match select_cell(catalog, remaining, used_pieces) {
        Some(cell) => cell,
        None => return false,
    };

    for &entry_index in &catalog.cell_to_entries[pivot] {
        let entry = &catalog.entries[entry_index];
        if used_pieces[entry.piece_index] {
            continue;
        }
        if entry.cell_indices.iter().any(|&cell| !remaining[cell]) {
            continue;
        }

        let piece = game.pieces[entry.piece_index].clone();
        let pip_order = piece.pips().to_vec();
        let placement = Placement::new(piece, entry.anchor, entry.orientation_index, pip_order);

        let next_constraints = match reduce_constraints(constraints.as_slice(), &placement) {
            Ok(result) => result,
            Err(_) => continue,
        };

        for &cell in &entry.cell_indices {
            remaining[cell] = false;
        }
        used_pieces[entry.piece_index] = true;
        *cells_remaining -= entry.cell_indices.len();

        let previous_constraints = mem::replace(constraints, next_constraints);
        placements.push(placement);

        if search(
            game,
            catalog,
            remaining,
            used_pieces,
            constraints,
            placements,
            cells_remaining,
        ) {
            return true;
        }

        placements.pop();
        *constraints = previous_constraints;
        *cells_remaining += entry.cell_indices.len();
        used_pieces[entry.piece_index] = false;
        for &cell in &entry.cell_indices {
            remaining[cell] = true;
        }
    }

    false
}

fn select_cell(
    catalog: &PlacementCatalog,
    remaining: &[bool],
    used_pieces: &[bool],
) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_count = usize::MAX;
    for (cell_index, &available) in remaining.iter().enumerate() {
        if !available {
            continue;
        }
        let mut count = 0;
        for &entry_index in &catalog.cell_to_entries[cell_index] {
            let entry = &catalog.entries[entry_index];
            if used_pieces[entry.piece_index] {
                continue;
            }
            if entry.cell_indices.iter().all(|&cell| remaining[cell]) {
                count += 1;
            }
        }
        if count == 0 {
            return None;
        }
        if count < best_count {
            best = Some(cell_index);
            best_count = count;
            if count == 1 {
                break;
            }
        }
    }
    best
}

fn validate_solution(game: &Game, placements: &[Placement]) -> Result<(), String> {
    let mut remaining: HashSet<Point> = game.board.to_hash_set();
    let mut used: HashMap<Point, Placement> = HashMap::new();
    let mut constraints: Vec<Constraint> = game.constraints.clone();

    for placement in placements {
        for point in placement.points() {
            if !remaining.remove(&point) {
                if let Some(prev) = used.get(&point) {
                    return Err(format!(
                        "cell {} already covered by {} while placing {}",
                        point, prev, placement
                    ));
                }
                return Err(format!("cell {} already covered", point));
            }
            used.insert(point, placement.clone());
        }

        constraints = reduce_constraints(&constraints, placement)?;
    }

    if !remaining.is_empty() {
        return Err("tiling did not cover entire board".to_string());
    }
    if !constraints.is_empty() {
        return Err("constraints not fully satisfied".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::solve;
    use crate::loader;
    use crate::model::{Constraint, Game, Piece, Pips, Point};
    use std::collections::HashSet;
    use std::path::Path;
    use std::sync::Arc;

    fn fixture(path: &str) -> String {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(path)
            .to_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn solves_domino_board() {
        let game =
            loader::load_game_from_path(fixture("poly_games/2x2.txt")).expect("load 2x2 game");
        assert!(solve(&game).is_ok());
    }

    #[test]
    fn solves_with_exact_sum_constraint() {
        let mut points = HashSet::new();
        points.insert(Point::new(0, 0));
        points.insert(Point::new(1, 0));
        let board = crate::model::Board::new(points.clone());
        let piece = Piece::domino(Pips::new(2).unwrap(), Pips::new(3).unwrap());
        let constraint = Constraint::Exactly {
            target: 5,
            points: Arc::new(points.clone()),
        };
        let game = Game::new(board, vec![piece], vec![constraint]);
        game.validate().expect("game should validate");
        let solution = solve(&game).expect("should find constrained solution");
        assert_eq!(solution.len(), 1);
    }

    #[test]
    fn detects_unsatisfied_constraint_early() {
        let mut points = HashSet::new();
        points.insert(Point::new(0, 0));
        points.insert(Point::new(1, 0));
        let board = crate::model::Board::new(points.clone());
        let piece = Piece::domino(Pips::new(1).unwrap(), Pips::new(1).unwrap());
        let constraint = Constraint::Exactly {
            target: 3,
            points: Arc::new(points),
        };
        let game = Game::new(board, vec![piece], vec![constraint]);
        game.validate().expect("game should validate");
        assert!(solve(&game).is_err());
    }
}
