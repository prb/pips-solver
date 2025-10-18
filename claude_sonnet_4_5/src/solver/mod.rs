// Solver module
// Implements the backtracking algorithm to solve games

use crate::data_model::*;
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

    // Find pivot point (prioritizes smallest constraint's top-left point)
    let pivot = game
        .pivot_point()
        .ok_or_else(|| "No valid placements.".to_string())?;

    // Get unique pieces
    let unique_pieces = game.unique_pieces();

    // Try each unique piece in each direction and anchor
    for piece in unique_pieces {
        let directions = if piece.is_doubleton() {
            // For doubletons, only try North and East (South and West are equivalent)
            vec![Direction::North, Direction::East]
        } else {
            vec![Direction::North, Direction::East, Direction::South, Direction::West]
        };

        for direction in directions {
            // Try multiple anchor points that could cover the pivot
            for anchor in anchors_for_direction(pivot, direction) {
                let placement = Placement::new(piece, anchor, direction);

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
    }

    // No valid placement found
    Err("No valid placements.".to_string())
}

/// Returns anchor points that could place a piece covering the pivot point.
/// For each direction, returns up to 2 anchor points where placing a piece
/// in that direction would cover the pivot.
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
            // North places top piece at (x, y+1), bottom piece at (x, y)
            // To cover pivot, either pivot is the bottom (anchor = (pivot.x, pivot.y-1))
            // or pivot is the top (anchor = pivot)
            push_unique(pivot.y.checked_sub(1).map(|y| Point::new(pivot.x, y)));
            push_unique(Some(pivot));
        }
        Direction::East => {
            // East places left piece at (x, y), right piece at (x+1, y)
            // To cover pivot, either pivot is the left (anchor = pivot)
            // or pivot is the right (anchor = (pivot.x-1, pivot.y))
            push_unique(Some(pivot));
            push_unique(pivot.x.checked_sub(1).map(|x| Point::new(x, pivot.y)));
        }
        Direction::South => {
            // South places top piece at (x, y), bottom piece at (x, y+1)
            // To cover pivot, either pivot is the top (anchor = pivot)
            // or pivot is the bottom (anchor = (pivot.x, pivot.y-1))
            push_unique(Some(pivot));
            push_unique(pivot.y.checked_sub(1).map(|y| Point::new(pivot.x, y)));
        }
        Direction::West => {
            // West places left piece at (x+1, y), right piece at (x, y)
            // To cover pivot, either pivot is the right (anchor = pivot)
            // or pivot is the left (anchor = (pivot.x-1, pivot.y))
            push_unique(Some(pivot));
            push_unique(pivot.x.checked_sub(1).map(|x| Point::new(x, pivot.y)));
        }
    }

    anchors
}
