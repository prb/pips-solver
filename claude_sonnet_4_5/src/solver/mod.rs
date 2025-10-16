// Solver module
// Implements the backtracking algorithm to solve games

use crate::data_model::*;
use std::collections::HashSet;
use std::rc::Rc;

// A persistent, singly-linked list of placements.
// Rc allows us to share the tail of the list between paths.
enum Path {
    Empty,
    Node(Placement, Rc<Path>),
}

impl Path {
    /// Convert the path to a Vec when a solution is found
    fn to_vec(&self) -> Vec<Placement> {
        let mut vec = Vec::new();
        let mut current = self;
        while let Path::Node(placement, next) = current {
            vec.push(*placement);
            current = next;
        }
        vec.reverse(); // The list is built backwards, so reverse it
        vec
    }
}

pub fn solve(game: Game) -> Result<Vec<Placement>, String> {
    let initial_path = Rc::new(Path::Empty);
    solve_recursive(game, initial_path).map(|path| path.to_vec())
}

fn solve_recursive(game: Game, path: Rc<Path>) -> Result<Rc<Path>, String> {
    // Base case: game is won
    if game.is_won() {
        return Ok(path);
    }

    // Find the upper-most, left-most point in the board
    let b0 = find_first_point(&game.board)?;

    // Get unique pieces
    let unique_pieces = get_unique_pieces(&game.pieces);

    // Try each unique piece in each direction
    for piece in unique_pieces {
        let directions = if piece.is_doubleton() {
            // For doubletons, only try North and East (South and West are equivalent)
            vec![Direction::North, Direction::East]
        } else {
            vec![Direction::North, Direction::East, Direction::South, Direction::West]
        };

        for direction in directions {
            let placement = Placement::new(piece, b0, direction);

            // Try to play this placement
            if let Ok(new_game) = game.play(&placement) {
                // Build new path by prepending this placement
                let new_path = Rc::new(Path::Node(placement, Rc::clone(&path)));

                // Recursively solve from the new game state
                if let Ok(solution) = solve_recursive(new_game, new_path) {
                    return Ok(solution);
                }
                // If this path didn't work, backtrack and try the next option
            }
        }
    }

    // No valid placement found
    Err("No valid placements.".to_string())
}

/// Finds the upper-most, left-most point in the board
fn find_first_point(board: &Board) -> Result<Point, String> {
    board
        .points()
        .iter()
        .min_by_key(|p| (p.y, p.x))
        .copied()
        .ok_or("Board is empty".to_string())
}

/// Gets the unique pieces from a list (removing duplicates)
fn get_unique_pieces(pieces: &[Piece]) -> Vec<Piece> {
    let unique: HashSet<Piece> = pieces.iter().copied().collect();
    unique.into_iter().collect()
}
