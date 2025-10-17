use crate::model::{Direction, Game, Piece, Placement, Point, reduce_constraints, remove_one};
use std::rc::Rc;

pub fn solve(game: &Game) -> Result<Vec<Placement>, String> {
    let seed = Rc::new(Path::Empty);
    backtrack(game, seed).map(|path| path_to_vec(&path))
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
    use super::solve;
    use crate::model::{Board, Direction, Game, Piece, Pips, Point};
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
}
