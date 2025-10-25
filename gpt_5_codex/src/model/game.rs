use super::{
    board::{Board, EMPTY_BOARD},
    constraint::ConstraintSet,
    piece::Piece,
    point::Point,
};
use once_cell::sync::Lazy;
use std::collections::HashSet;

/// Represents a full game state, including remaining board points, pieces, and constraints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Game {
    pub board: Board,
    pub pieces: Vec<Piece>,
    pub constraints: ConstraintSet,
}

impl Game {
    pub fn new(board: Board, pieces: Vec<Piece>, constraints: ConstraintSet) -> Self {
        Self {
            board,
            pieces,
            constraints,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let total_cells: usize = self
            .pieces
            .iter()
            .map(|piece| piece.shape().cell_count())
            .sum();
        if self.board.len() != total_cells {
            return Err(
                "Board must have the same number of points as the total cells across pieces."
                    .to_string(),
            );
        }

        let mut seen_points: HashSet<Point> = HashSet::new();
        let board_points = self.board.points();
        for constraint in &self.constraints {
            constraint.validate()?;
            for point in constraint.points() {
                if !board_points.contains(point) {
                    return Err(format!(
                        "Constraint references point ({}, {}) that is not on the board.",
                        point.x, point.y
                    ));
                }
                if !seen_points.insert(*point) {
                    return Err(format!(
                        "Point ({}, {}) appears in more than one constraint.",
                        point.x, point.y
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn is_won(&self) -> bool {
        self.board.is_empty() && self.pieces.is_empty() && self.constraints.is_empty()
    }

    pub fn pivot_point(&self) -> Option<Point> {
        let board_points = self.board.points();

        if let Some((point, _size)) = self
            .constraints
            .iter()
            .filter_map(|constraint| {
                let points = constraint.points();
                if points.is_empty() {
                    None
                } else {
                    let mut sorted: Vec<Point> = points
                        .iter()
                        .copied()
                        .filter(|p| board_points.contains(p))
                        .collect();
                    if sorted.is_empty() {
                        return None;
                    }
                    sorted.sort_by_key(|p| (p.y, p.x));
                    Some((sorted[0], points.len()))
                }
            })
            .min_by_key(|&(point, size)| (size, point.x, point.y))
        {
            return Some(point);
        }

        self.board
            .points()
            .iter()
            .min_by_key(|point| (point.y, point.x))
            .copied()
    }

    pub fn unique_pieces(&self) -> Vec<Piece> {
        let mut unique = HashSet::new();
        let mut list = Vec::new();
        for piece in &self.pieces {
            if unique.insert(piece.clone()) {
                list.push(piece.clone());
            }
        }
        list
    }
}

#[allow(dead_code)]
pub static WON_GAME: Lazy<Game> = Lazy::new(|| Game {
    board: EMPTY_BOARD.clone(),
    pieces: Vec::new(),
    constraints: Vec::new(),
});

#[cfg(test)]
mod tests {
    use super::Game;
    use crate::model::{
        board::Board, constraint::Constraint, piece::Piece, pips::Pips, point::Point,
    };
    use std::collections::HashSet;

    #[test]
    fn validation_checks_board_piece_ratio() {
        let board = Board::default();
        let game = Game::new(board.clone(), vec![], vec![]);
        assert!(game.validate().is_ok());

        let mut points = HashSet::new();
        points.insert(Point::new(0, 0));
        let board = Board::new(points);
        let piece = Piece::domino(Pips::new(0).unwrap(), Pips::new(0).unwrap());
        let game = Game::new(board, vec![piece], vec![]);
        assert!(game.validate().is_err());
    }

    #[test]
    fn validation_fails_when_constraint_points_overlap() {
        let mut board_points = HashSet::new();
        board_points.insert(Point::new(0, 0));
        board_points.insert(Point::new(1, 0));
        let board = Board::new(board_points);

        let piece = Piece::domino(Pips::new(0).unwrap(), Pips::new(0).unwrap());

        let mut c_points = HashSet::new();
        c_points.insert(Point::new(0, 0));

        let constraints = vec![
            Constraint::Exactly {
                target: 0,
                points: c_points.clone(),
            },
            Constraint::LessThan {
                target: 5,
                points: c_points,
            },
        ];

        let game = Game::new(board, vec![piece], constraints);
        assert!(game.validate().is_err());
    }

    #[test]
    fn validation_fails_when_constraint_points_not_on_board() {
        let mut board_points = HashSet::new();
        board_points.insert(Point::new(0, 0));
        board_points.insert(Point::new(1, 0));
        let board = Board::new(board_points);

        let piece = Piece::domino(Pips::new(0).unwrap(), Pips::new(0).unwrap());

        let mut c_points = HashSet::new();
        c_points.insert(Point::new(2, 0)); // not on board

        let constraints = vec![Constraint::Exactly {
            target: 0,
            points: c_points,
        }];

        let game = Game::new(board, vec![piece], constraints);
        assert!(game.validate().is_err());
    }
}
