use crate::model::{Direction, Game, Piece, Placement, reduce_constraints, remove_one};

pub fn solve(game: &Game) -> Result<Vec<Placement>, String> {
    let mut path = Vec::new();
    backtrack(game.clone(), &mut path)
}

fn backtrack(game: Game, path: &mut Vec<Placement>) -> Result<Vec<Placement>, String> {
    if game.is_won() {
        return Ok(path.clone());
    }
    let pivot = match game.top_left_point() {
        Some(point) => point,
        None => return Err("No valid placements.".to_string()),
    };
    let unique_pieces = game.unique_pieces();
    for piece in unique_pieces {
        for direction in directions_for_piece(&piece) {
            let placement = Placement::new(piece.clone(), pivot, direction);
            match play(&game, &placement) {
                Ok(next) => {
                    path.push(placement.clone());
                    match backtrack(next, path) {
                        Ok(solution) => return Ok(solution),
                        Err(_) => {
                            path.pop();
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    }
    Err("No valid placements.".to_string())
}

fn play(game: &Game, placement: &Placement) -> Result<Game, String> {
    let board_result = game.board.remove_points(&placement.points());
    let pieces_result = remove_one(game.pieces.clone(), &placement.piece);
    let constraints_result = reduce_constraints(&game.constraints, placement);

    match (board_result, pieces_result, constraints_result) {
        (Ok(board), Ok(pieces), Ok(constraints)) => Ok(Game::new(board, pieces, constraints)),
        _ => Err("Unwinnable game.".to_string()),
    }
}

fn directions_for_piece(piece: &Piece) -> Vec<Direction> {
    if piece.is_doubleton() {
        vec![Direction::North, Direction::East]
    } else {
        vec![
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ]
    }
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
