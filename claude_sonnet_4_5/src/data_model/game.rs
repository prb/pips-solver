// Game - represents a complete game state

use super::board::Board;
use super::constraint::{reduce_cs, Constraint};
use super::piece;
use super::piece::Piece;
use super::placement::Placement;
use super::point::Point;
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

    /// Finds a pivot point for placing the next piece.
    /// Prioritizes points from the smallest constraint to avoid expensive backtracking.
    /// Falls back to the top-left point of the board if no constraints exist.
    pub fn pivot_point(&self) -> Option<Point> {
        let board_points = self.board.points();

        // Find the smallest constraint's top-left point on the board
        if let Some((point, _size)) = self
            .constraints
            .iter()
            .filter_map(|constraint| {
                let points = match constraint {
                    Constraint::Empty => return None,
                    Constraint::AllSame { points, .. } => points,
                    Constraint::AllDifferent { points, .. } => points,
                    Constraint::LessThan { points, .. } => points,
                    Constraint::Exactly { points, .. } => points,
                    Constraint::MoreThan { points, .. } => points,
                };

                if points.is_empty() {
                    return None;
                }

                // Find points that are still on the board
                let mut sorted: Vec<Point> = points
                    .iter()
                    .copied()
                    .filter(|p| board_points.contains(p))
                    .collect();

                if sorted.is_empty() {
                    return None;
                }

                // Sort by (y, x) to get top-left
                sorted.sort_by_key(|p| (p.y, p.x));
                Some((sorted[0], points.len()))
            })
            .min_by_key(|&(point, size)| (size, point.x, point.y))
        {
            return Some(point);
        }

        // Fall back to board's top-left point
        board_points.iter().min_by_key(|p| (p.y, p.x)).copied()
    }

    /// Gets unique pieces from the piece list, preserving order
    pub fn unique_pieces(&self) -> Vec<Piece> {
        let mut seen = HashSet::new();
        let mut unique = Vec::new();
        for &piece in &self.pieces {
            if seen.insert(piece) {
                unique.push(piece);
            }
        }
        unique
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
