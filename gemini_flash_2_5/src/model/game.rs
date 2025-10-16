use crate::model::board::Board;
use crate::model::constraint::Constraint;
use crate::model::piece::Piece;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    pub board: Board,
    pub pieces: Vec<Piece>,
    pub constraints: Vec<Constraint>,
}

impl Game {
    pub fn new(board: Board, pieces: Vec<Piece>, constraints: Vec<Constraint>) -> Self {
        Game {
            board,
            pieces,
            constraints,
        }
    }

    pub fn is_valid(&self) -> bool {
        let board_points = self.board.points().len();
        let pieces_len = self.pieces.len();
        let mut constraint_points = HashSet::new();
        for c in &self.constraints {
            let points = match c {
                Constraint::AllSame(_, p) => p,
                Constraint::AllDifferent(_, p) => p,
                Constraint::LessThan(_, p) => p,
                Constraint::Exactly(_, p) => p,
                Constraint::MoreThan(_, p) => p,
            };
            for point in points {
                if !constraint_points.insert(point) {
                    return false; // Overlapping points in constraints
                }
            }
        }
        board_points == 2 * pieces_len
    }

    pub fn is_won(&self) -> bool {
        self.board.points().is_empty() && self.pieces.is_empty() && self.constraints.is_empty()
    }
}

pub fn won_game() -> Game {
    Game {
        board: Board::new(HashSet::new()),
        pieces: Vec::new(),
        constraints: Vec::new(),
    }
}
