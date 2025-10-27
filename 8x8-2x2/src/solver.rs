use crate::board::Board;
use crate::polyomino::{Pentomino, Placement, generate_orientations};

pub fn solve(board: &Board) -> Option<Vec<Placement>> {
    let pentominoes = Pentomino::all();
    let mut solution = Vec::new();
    let mut board = board.clone();

    if backtrack(&mut board, &pentominoes, &mut solution) {
        Some(solution)
    } else {
        None
    }
}

fn backtrack(
    board: &mut Board,
    remaining_pentominoes: &[Pentomino],
    solution: &mut Vec<Placement>,
) -> bool {
    // Base case: all pentominoes placed
    if remaining_pentominoes.is_empty() {
        return board.available_cells().is_empty();
    }

    // Get the first available cell - this is the cell we must cover
    let Some((target_x, target_y)) = board.first_available() else {
        return false;
    };

    // Try each remaining pentomino at this position
    for (idx, &pentomino) in remaining_pentominoes.iter().enumerate() {
        let base_shape = pentomino.base_shape();
        let orientations = generate_orientations(&base_shape);

        // Try each orientation
        for orientation in &orientations {
            // For each cell in this orientation, try placing the piece so that cell covers target
            for &(dx, dy) in orientation.iter() {
                // Calculate offset so that this cell lands on target
                let offset_x = target_x as i32 - dx;
                let offset_y = target_y as i32 - dy;

                // Convert all cells to board coordinates
                let cells: Result<Vec<(usize, usize)>, _> = orientation
                    .iter()
                    .map(|&(dx2, dy2)| {
                        let x = offset_x + dx2;
                        let y = offset_y + dy2;
                        if x >= 0 && y >= 0 && x < board.width as i32 && y < board.height as i32 {
                            Ok((x as usize, y as usize))
                        } else {
                            Err(())
                        }
                    })
                    .collect();

                let Ok(cells) = cells else {
                    continue;
                };

                // Try to place the piece
                if board.place(&cells) {
                    solution.push(Placement {
                        pentomino,
                        cells: cells.clone(),
                    });

                    // Create list of remaining pentominoes (without this one)
                    let mut new_remaining = remaining_pentominoes.to_vec();
                    new_remaining.remove(idx);

                    // Recurse
                    if backtrack(board, &new_remaining, solution) {
                        return true;
                    }

                    // Backtrack
                    solution.pop();
                    board.unplace(&cells);
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polyomino::Pentomino;

    #[test]
    fn test_can_place_single_piece() {
        let mut board = Board::new(8, 8, 0, 0);
        let pentomino = Pentomino::F;
        let base_shape = pentomino.base_shape();
        let orientations = generate_orientations(&base_shape);

        // Try to place F at position (2, 0)
        let cells: Vec<(usize, usize)> = orientations[0]
            .iter()
            .map(|&(dx, dy)| ((2 + dx) as usize, (0 + dy) as usize))
            .collect();

        let placed = board.place(&cells);
        assert!(placed, "Should be able to place F pentomino");
    }

    #[test]
    fn test_simple_board() {
        let board = Board::new(8, 8, 0, 0);
        let solution = solve(&board);
        assert!(solution.is_some(), "Should find a solution for hole at (0,0)");

        if let Some(sol) = solution {
            assert_eq!(sol.len(), 12, "Solution should have 12 pentominoes");

            // Check that all cells are covered exactly once
            let mut covered_cells = std::collections::HashSet::new();
            for placement in &sol {
                for &cell in &placement.cells {
                    assert!(covered_cells.insert(cell), "Cell {:?} covered twice", cell);
                }
            }
            assert_eq!(covered_cells.len(), 60, "Should cover exactly 60 cells");
        }
    }
}
