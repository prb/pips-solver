// Game - represents a complete game state

use super::board::Board;
use super::constraint::{reduce_cs, Constraint};
use super::piece;
use super::piece::Piece;
use super::placement::Placement;
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

    // pub fn won() -> Self {
    //     Game {
    //         board: Board::empty(),
    //         pieces: Vec::new(),
    //         constraints: Vec::new(),
    //     }
    // }

    pub fn is_won(&self) -> bool {
        self.board.is_empty() && self.pieces.is_empty() && self.constraints.is_empty()
    }

    pub fn is_valid(&self) -> bool {
        // Check if number of board points equals double the number of pieces
        if self.board.points().len() != self.pieces.len() * 2 {
            return false;
        }

        // Check if constraints are consistent (no point appears in multiple constraints)
        let mut all_constraint_points = HashSet::new();
        for constraint in &self.constraints {
            let constraint_points = match constraint {
                Constraint::Empty => HashSet::new(),
                Constraint::AllSame { points, .. } => points.clone(),
                Constraint::AllDifferent { points, .. } => points.clone(),
                Constraint::LessThan { points, .. } => points.clone(),
                Constraint::Exactly { points, .. } => points.clone(),
                Constraint::MoreThan { points, .. } => points.clone(),
            };

            for point in constraint_points {
                if all_constraint_points.contains(&point) {
                    return false; // Point appears in multiple constraints
                }
                all_constraint_points.insert(point);
            }
        }

        true
    }

    /// Applies a placement to the game, returning a new game state (play from spec)
    pub fn play(&self, placement: &Placement) -> Result<Self, String> {
        // Reduce the board
        let new_board = self.board.reduce_b(placement)?;

        // Remove the piece from pieces
        let new_pieces = piece::remove_one(self.pieces.clone(), placement.piece)?;

        // Reduce the constraints
        let new_constraints = reduce_cs(&self.constraints, placement)?;

        Ok(Game::new(new_board, new_pieces, new_constraints))
    }
}
