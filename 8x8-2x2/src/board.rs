use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    available: HashSet<(usize, usize)>,
}

impl Board {
    pub fn new(width: usize, height: usize, hole_x: usize, hole_y: usize) -> Self {
        let mut available = HashSet::new();

        // Add all cells except the 2x2 hole
        for y in 0..height {
            for x in 0..width {
                // Skip cells that are part of the 2x2 hole
                if x >= hole_x && x < hole_x + 2 && y >= hole_y && y < hole_y + 2 {
                    continue;
                }
                available.insert((x, y));
            }
        }

        Board {
            width,
            height,
            available,
        }
    }

    pub fn available_cells(&self) -> &HashSet<(usize, usize)> {
        &self.available
    }

    pub fn place(&mut self, cells: &[(usize, usize)]) -> bool {
        // Check if all cells are available
        for &cell in cells {
            if !self.available.contains(&cell) {
                return false;
            }
        }

        // Remove cells from available
        for &cell in cells {
            self.available.remove(&cell);
        }

        true
    }

    pub fn unplace(&mut self, cells: &[(usize, usize)]) {
        for &cell in cells {
            self.available.insert(cell);
        }
    }

    pub fn first_available(&self) -> Option<(usize, usize)> {
        // Return cells in order (top to bottom, left to right) for consistent solving
        for y in 0..self.height {
            for x in 0..self.width {
                if self.available.contains(&(x, y)) {
                    return Some((x, y));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_creation() {
        let board = Board::new(8, 8, 0, 0);
        assert_eq!(board.available_cells().len(), 60); // 64 - 4
    }

    #[test]
    fn test_board_with_different_hole() {
        let board = Board::new(8, 8, 3, 3);
        assert_eq!(board.available_cells().len(), 60);
        assert!(!board.available_cells().contains(&(3, 3)));
        assert!(!board.available_cells().contains(&(4, 3)));
        assert!(!board.available_cells().contains(&(3, 4)));
        assert!(!board.available_cells().contains(&(4, 4)));
        assert!(board.available_cells().contains(&(2, 2)));
        assert!(board.available_cells().contains(&(5, 5)));
    }
}
