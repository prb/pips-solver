use crate::model::{Game, Placement, Point, reduce_constraints, remove_one};
use std::rc::Rc;

pub fn solve(game: &Game) -> Result<Vec<Placement>, String> {
    let seed = Rc::new(Path::Empty);
    backtrack(game, seed).map(|path| path_to_vec(&path))
}

pub fn count_solutions(game: &Game) -> Result<usize, String> {
    let mut total = 0usize;
    explore(game, &mut total)?;
    Ok(total)
}

fn backtrack(game: &Game, path: Rc<Path>) -> Result<Rc<Path>, String> {
    if game.is_won() {
        return Ok(path);
    }
    let pivot = game
        .pivot_point()
        .ok_or_else(|| "No valid placements.".to_string())?;
    let unique_pieces = game.unique_pieces();
    for piece in unique_pieces {
        let permutations = piece.pip_permutations();
        for (orientation_index, offsets) in piece.orientations().iter().enumerate() {
            for anchor in anchors_for_orientation(pivot, offsets) {
                for pip_order in &permutations {
                    let placement =
                        Placement::new(piece.clone(), anchor, orientation_index, pip_order.clone());
                    match play(game, &placement) {
                        Ok(next) => {
                            let next_path = Rc::new(Path::Node {
                                placement,
                                prev: Rc::clone(&path),
                            });
                            if let Ok(solution) = backtrack(&next, next_path) {
                                return Ok(solution);
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
        }
    }
    Err("No valid placements.".to_string())
}

fn explore(game: &Game, total: &mut usize) -> Result<(), String> {
    if game.is_won() {
        *total += 1;
        return Ok(());
    }
    let pivot = match game.pivot_point() {
        Some(point) => point,
        None => return Ok(()),
    };
    let unique_pieces = game.unique_pieces();
    for piece in unique_pieces {
        let permutations = piece.pip_permutations();
        for (orientation_index, offsets) in piece.orientations().iter().enumerate() {
            for anchor in anchors_for_orientation(pivot, offsets) {
                for pip_order in &permutations {
                    let placement =
                        Placement::new(piece.clone(), anchor, orientation_index, pip_order.clone());
                    match play(game, &placement) {
                        Ok(next) => {
                            explore(&next, total)?;
                        }
                        Err(_) => continue,
                    }
                }
            }
        }
    }
    Ok(())
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

fn anchors_for_orientation(pivot: Point, offsets: &[(i32, i32)]) -> Vec<Point> {
    let mut anchors = Vec::new();
    for &(dx, dy) in offsets {
        if let Some(anchor) = pivot_anchor(pivot, dx, dy) {
            if !anchors.contains(&anchor) {
                anchors.push(anchor);
            }
        }
    }
    anchors
}

fn pivot_anchor(pivot: Point, dx: i32, dy: i32) -> Option<Point> {
    let anchor_x = pivot.x as i32 - dx;
    let anchor_y = pivot.y as i32 - dy;
    if anchor_x < 0 || anchor_y < 0 {
        return None;
    }
    Some(Point::new(anchor_x as u32, anchor_y as u32))
}

enum Path {
    Empty,
    Node {
        placement: Placement,
        prev: Rc<Path>,
    },
}

fn path_to_vec(path: &Path) -> Vec<Placement> {
    let mut current = path;
    let mut placements = Vec::new();
    while let Path::Node { placement, prev } = current {
        placements.push(placement.clone());
        current = prev;
    }
    placements.reverse();
    placements
}

#[cfg(test)]
mod tests {
    use super::{count_solutions, solve};
    use crate::model::{Board, Constraint, Game, Piece, Pips, Point, PolyShape};
    use std::collections::HashSet;

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
            points: constraint_points,
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
            PolyShape::I3,
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
