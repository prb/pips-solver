use crate::model::board::Board;
use crate::model::constraint::Constraint;
use crate::model::game::Game;
use crate::model::piece::{self, Piece};
use crate::model::placement::Placement;
use std::collections::HashSet;

pub fn solve(game: &Game) -> Result<Vec<Placement>, String> {
    solve_recursive(game, Vec::new())
}

use crate::model::direction::Direction;
fn solve_recursive(game: &Game, placements: Vec<Placement>) -> Result<Vec<Placement>, String> {
    if game.is_won() {
        return Ok(placements);
    }

    let b0 = match game.board.points().iter().min() {
        Some(p) => *p,
        None => return Err("Board is empty, but game is not won".to_string()),
    };

    let unique_pieces: HashSet<Piece> = game.pieces.iter().cloned().collect();

    for piece in unique_pieces {
        for direction in &[
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            if piece.is_doubleton()
                && (*direction == Direction::South || *direction == Direction::West)
            {
                continue;
            }

            let placement = Placement {
                piece,
                point: b0,
                direction: *direction,
            };

            if let Ok(new_game) = play(game, &placement) {
                let mut new_placements = placements.clone();
                new_placements.push(placement);
                if let Ok(solution) = solve_recursive(&new_game, new_placements) {
                    return Ok(solution);
                }
            }
        }
    }

    Err("No solution found".to_string())
}

fn play(game: &Game, placement: &Placement) -> Result<Game, String> {
    let new_board = reduce_b(&game.board, placement)?;
    let new_pieces = piece::remove_one(&game.pieces, &placement.piece)?;
    let new_constraints = reduce_cs(&game.constraints, placement)?;

    Ok(Game::new(new_board, new_pieces, new_constraints))
}

fn reduce_b(board: &Board, placement: &Placement) -> Result<Board, String> {
    let placement_points = placement.points();
    if !placement_points.is_subset(board.points()) {
        return Err("Placement has points outside of the board".to_string());
    }
    let new_points = board
        .points()
        .difference(&placement_points)
        .cloned()
        .collect();
    Ok(Board::new(new_points))
}

fn reduce_cs(constraints: &[Constraint], placement: &Placement) -> Result<Vec<Constraint>, String> {
    let mut new_constraints = Vec::new();
    for constraint in constraints {
        let new_constraint = reduce_p(constraint, placement)?;
        if let Some(c) = new_constraint {
            new_constraints.push(c);
        }
    }
    Ok(new_constraints)
}

fn reduce_p(constraint: &Constraint, placement: &Placement) -> Result<Option<Constraint>, String> {
    let assignments = placement.assignments();
    let mut current_constraint = Some(constraint.clone());

    for assignment in &assignments {
        if let Some(c) = current_constraint {
            current_constraint = c.reduce_a(assignment)?;
        } else {
            break;
        }
    }
    Ok(current_constraint)
}
