use super::point::Point;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::Arc;

/// Represents the playable board as a bitset within a bounding box.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    storage: Arc<BoardStorage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BoardStorage {
    min_x: u32,
    min_y: u32,
    width: u32,
    height: u32,
    bits: Vec<u64>,
    len: usize,
}

impl BoardStorage {
    fn empty() -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            width: 0,
            height: 0,
            bits: Vec::new(),
            len: 0,
        }
    }

    fn total_cells(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    fn normalize_point(&self, point: &Point) -> Option<(usize, u32, u32)> {
        if self.len == 0 {
            return None;
        }
        if point.x < self.min_x
            || point.y < self.min_y
            || point.x >= self.min_x + self.width
            || point.y >= self.min_y + self.height
        {
            return None;
        }
        let rel_x = point.x - self.min_x;
        let rel_y = point.y - self.min_y;
        let index = (rel_y * self.width + rel_x) as usize;
        Some((index, rel_x, rel_y))
    }

    fn bit_index(&self, index: usize) -> (usize, u32) {
        let word = index / 64;
        let offset = (index % 64) as u32;
        (word, offset)
    }

    fn test_bit(&self, index: usize) -> bool {
        let (word, offset) = self.bit_index(index);
        if word >= self.bits.len() {
            return false;
        }
        (self.bits[word] & (1u64 << offset)) != 0
    }

    fn clear_bit(&mut self, index: usize) {
        let (word, offset) = self.bit_index(index);
        if word < self.bits.len() && (self.bits[word] & (1u64 << offset)) != 0 {
            self.bits[word] &= !(1u64 << offset);
            self.len -= 1;
        }
    }
}

impl Board {
    pub fn new(points: HashSet<Point>) -> Self {
        if points.is_empty() {
            return Self {
                storage: Arc::new(BoardStorage::empty()),
            };
        }

        let min_x = points.iter().map(|p| p.x).min().unwrap();
        let max_x = points.iter().map(|p| p.x).max().unwrap();
        let min_y = points.iter().map(|p| p.y).min().unwrap();
        let max_y = points.iter().map(|p| p.y).max().unwrap();

        let width = max_x - min_x + 1;
        let height = max_y - min_y + 1;
        let total_cells = (width as usize) * (height as usize);
        let mut bits = vec![0u64; (total_cells + 63) / 64];

        for point in &points {
            let rel_x = point.x - min_x;
            let rel_y = point.y - min_y;
            let index = (rel_y * width + rel_x) as usize;
            let (word, offset) = (index / 64, (index % 64) as u32);
            bits[word] |= 1u64 << offset;
        }

        let storage = BoardStorage {
            min_x,
            min_y,
            width,
            height,
            bits,
            len: points.len(),
        };

        Self {
            storage: Arc::new(storage),
        }
    }

    pub fn len(&self) -> usize {
        self.storage.len
    }

    pub fn is_empty(&self) -> bool {
        self.storage.len == 0
    }

    pub fn bounds(&self) -> Option<(u32, u32, u32, u32)> {
        if self.is_empty() {
            None
        } else {
            Some((
                self.storage.min_x,
                self.storage.max_x(),
                self.storage.min_y,
                self.storage.max_y(),
            ))
        }
    }

    pub fn contains_point(&self, point: &Point) -> bool {
        match self.storage.normalize_point(point) {
            Some((index, _, _)) => self.storage.test_bit(index),
            None => false,
        }
    }

    pub fn index_of(&self, point: &Point) -> Option<usize> {
        self.storage
            .normalize_point(point)
            .map(|(index, _, _)| index)
    }

    pub fn contains_all(&self, other: &[Point]) -> bool {
        other.iter().all(|point| self.contains_point(point))
    }

    pub fn contains_indices(&self, indices: &[usize]) -> bool {
        indices.iter().all(|&index| self.storage.test_bit(index))
    }

    pub fn total_cells(&self) -> usize {
        self.storage.total_cells()
    }

    pub fn remove_points(&self, to_remove: &[Point]) -> Result<Self, String> {
        if !self.contains_all(to_remove) {
            return Err("Placement has at least one point outside of the board.".to_string());
        }

        let mut storage = Arc::clone(&self.storage);
        let data = Arc::make_mut(&mut storage);
        for point in to_remove {
            if let Some((index, _, _)) = data.normalize_point(point) {
                data.clear_bit(index);
            }
        }
        Ok(Board { storage })
    }

    pub fn iter(&self) -> BoardIter<'_> {
        BoardIter {
            storage: &self.storage,
            index: 0,
        }
    }

    pub fn to_hash_set(&self) -> HashSet<Point> {
        self.iter().collect()
    }
}

impl BoardStorage {
    fn max_x(&self) -> u32 {
        if self.width == 0 {
            0
        } else {
            self.min_x + self.width - 1
        }
    }

    fn max_y(&self) -> u32 {
        if self.height == 0 {
            0
        } else {
            self.min_y + self.height - 1
        }
    }
}

pub struct BoardIter<'a> {
    storage: &'a BoardStorage,
    index: usize,
}

impl<'a> Iterator for BoardIter<'a> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let total = self.storage.total_cells();
        while self.index < total {
            let current = self.index;
            self.index += 1;
            if self.storage.test_bit(current) {
                let width = self.storage.width as usize;
                let rel_y = current / width;
                let rel_x = current % width;
                let x = self.storage.min_x + rel_x as u32;
                let y = self.storage.min_y + rel_y as u32;
                return Some(Point::new(x, y));
            }
        }
        None
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            storage: Arc::new(BoardStorage::empty()),
        }
    }
}

#[allow(dead_code)]
pub static EMPTY_BOARD: Lazy<Board> = Lazy::new(Board::default);

#[cfg(test)]
mod tests {
    use super::{Board, Point};
    use std::collections::HashSet;

    #[test]
    fn remove_points_succeeds_for_subset() {
        let mut pts = HashSet::new();
        pts.insert(Point::new(0, 0));
        pts.insert(Point::new(0, 1));
        let board = Board::new(pts.clone());

        let take = [Point::new(0, 1)];
        let next = board.remove_points(&take).unwrap();
        assert_eq!(next.len(), 1);
        assert!(next.contains_point(&Point::new(0, 0)));
    }

    #[test]
    fn remove_points_errors_for_non_subset() {
        let board = Board::default();
        let take = [Point::new(0, 0)];
        assert!(board.remove_points(&take).is_err());
    }
}
