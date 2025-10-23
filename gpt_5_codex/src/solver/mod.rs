use crate::model::{Direction, Game, Piece, Placement, Point, reduce_constraints, remove_one};
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
        for &direction in directions_for_piece(&piece) {
            for anchor in anchors_for_direction(pivot, direction) {
                let placement = Placement::new(piece.clone(), anchor, direction);
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
        for &direction in directions_for_piece(&piece) {
            for anchor in anchors_for_direction(pivot, direction) {
                let placement = Placement::new(piece.clone(), anchor, direction);
                match play(game, &placement) {
                    Ok(next) => {
                        explore(&next, total)?;
                    }
                    Err(_) => continue,
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

fn directions_for_piece(piece: &Piece) -> &'static [Direction] {
    if piece.is_doubleton() {
        &DOUBLETON_DIRECTIONS
    } else {
        &ALL_DIRECTIONS
    }
}

fn anchors_for_direction(pivot: Point, direction: Direction) -> Vec<Point> {
    let mut anchors = Vec::with_capacity(2);
    let mut push_unique = |opt: Option<Point>| {
        if let Some(point) = opt {
            if !anchors.contains(&point) {
                anchors.push(point);
            }
        }
    };

    match direction {
        Direction::North => {
            push_unique(pivot.y.checked_sub(1).map(|y| Point::new(pivot.x, y)));
            push_unique(Some(pivot));
        }
        Direction::East => {
            push_unique(Some(pivot));
            push_unique(pivot.x.checked_sub(1).map(|x| Point::new(x, pivot.y)));
        }
        Direction::South => {
            push_unique(Some(pivot));
            push_unique(pivot.y.checked_sub(1).map(|y| Point::new(pivot.x, y)));
        }
        Direction::West => {
            push_unique(Some(pivot));
            push_unique(pivot.x.checked_sub(1).map(|x| Point::new(x, pivot.y)));
        }
    }

    anchors
}

const ALL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

const DOUBLETON_DIRECTIONS: [Direction; 2] = [Direction::North, Direction::East];

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
    use crate::model::{Board, Constraint, Direction, Game, Piece, Pips, Point};
    use std::collections::HashSet;

    #[test]
    fn solves_single_piece_board() {
        let mut points = HashSet::new();
        points.insert(Point::new(0, 0));
        points.insert(Point::new(1, 0));
        let board = Board::new(points);
        let pieces = vec![Piece::new(Pips::new(1).unwrap(), Pips::new(1).unwrap())];
        let game = Game::new(board, pieces, vec![]);
        game.validate().unwrap();
        let solution = solve(&game).expect("solution should exist");
        assert_eq!(solution.len(), 1);
        assert_eq!(solution[0].direction, Direction::East);
    }

    #[test]
    fn counts_single_solution() {
        let mut points = HashSet::new();
        points.insert(Point::new(0, 0));
        points.insert(Point::new(1, 0));
        let board = Board::new(points);
        let pieces = vec![Piece::new(Pips::new(1).unwrap(), Pips::new(1).unwrap())];
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

        let pieces = vec![Piece::new(Pips::new(1).unwrap(), Pips::new(2).unwrap())];
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
}
