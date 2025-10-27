use super::{
    board::{Board, EMPTY_BOARD},
    constraint::ConstraintSet,
    piece::Piece,
    point::Point,
};
use once_cell::sync::Lazy;
use std::cmp::Ordering;
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
        for constraint in &self.constraints {
            constraint.validate()?;
            for point in constraint.points() {
                if !self.board.contains_point(point) {
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
        if self.board.is_empty() {
            return None;
        }

        let components = connected_components(&self.board);
        components.into_iter().find_map(|component| {
            if let Some(point) = self.constraint_pivot(&component) {
                Some(point)
            } else {
                Some(component.min_point)
            }
        })
    }

    fn constraint_pivot(&self, component: &BoardComponent) -> Option<Point> {
        self.constraints
            .iter()
            .filter_map(|constraint| {
                let mut relevant: Vec<Point> = constraint
                    .points()
                    .iter()
                    .copied()
                    .filter(|p| self.board.contains_point(p) && component.point_set.contains(p))
                    .collect();
                if relevant.is_empty() {
                    return None;
                }
                relevant.sort_by(|a, b| compare_points(*a, *b));
                let pivot = relevant[0];
                let slack = region_slack(&relevant);
                Some((pivot, relevant.len(), slack))
            })
            .min_by(|a, b| {
                a.1.cmp(&b.1)
                    .then_with(|| a.2.cmp(&b.2))
                    .then_with(|| compare_points(a.0, b.0))
            })
            .map(|(point, _, _)| point)
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
    use std::sync::Arc;

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
                points: Arc::new(c_points.clone()),
            },
            Constraint::LessThan {
                target: 5,
                points: Arc::new(c_points),
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
            points: Arc::new(c_points),
        }];

        let game = Game::new(board, vec![piece], constraints);
        assert!(game.validate().is_err());
    }
}

struct BoardComponent {
    points: Vec<Point>,
    point_set: HashSet<Point>,
    min_point: Point,
    slack: usize,
}

fn connected_components(board: &Board) -> Vec<BoardComponent> {
    let mut visited: HashSet<Point> = HashSet::new();
    let mut components = Vec::new();

    for start in board.iter() {
        if !visited.insert(start) {
            continue;
        }

        let mut stack = vec![start];
        let mut points = Vec::new();
        let mut point_set = HashSet::new();
        let mut min_point = start;
        let mut min_x = start.x;
        let mut max_x = start.x;
        let mut min_y = start.y;
        let mut max_y = start.y;

        while let Some(current) = stack.pop() {
            points.push(current);
            point_set.insert(current);
            if compare_points(current, min_point) == Ordering::Less {
                min_point = current;
            }
            if current.x < min_x {
                min_x = current.x;
            }
            if current.x > max_x {
                max_x = current.x;
            }
            if current.y < min_y {
                min_y = current.y;
            }
            if current.y > max_y {
                max_y = current.y;
            }

            for neighbor in orthogonal_neighbors(current) {
                if board.contains_point(&neighbor) && visited.insert(neighbor) {
                    stack.push(neighbor);
                }
            }
        }

        let bounding_area = ((max_x - min_x + 1) as usize) * ((max_y - min_y + 1) as usize);
        let slack = bounding_area - points.len();

        components.push(BoardComponent {
            points,
            point_set,
            min_point,
            slack,
        });
    }

    components.sort_by(|a, b| {
        a.points
            .len()
            .cmp(&b.points.len())
            .then_with(|| a.slack.cmp(&b.slack))
            .then_with(|| compare_points(a.min_point, b.min_point))
    });

    components
}

fn orthogonal_neighbors(point: Point) -> Vec<Point> {
    let mut neighbors = Vec::with_capacity(4);
    if let Some(x) = point.x.checked_sub(1) {
        neighbors.push(Point::new(x, point.y));
    }
    if let Some(x) = point.x.checked_add(1) {
        neighbors.push(Point::new(x, point.y));
    }
    if let Some(y) = point.y.checked_sub(1) {
        neighbors.push(Point::new(point.x, y));
    }
    if let Some(y) = point.y.checked_add(1) {
        neighbors.push(Point::new(point.x, y));
    }
    neighbors
}

fn compare_points(a: Point, b: Point) -> Ordering {
    a.y.cmp(&b.y).then_with(|| a.x.cmp(&b.x))
}

fn region_slack(points: &[Point]) -> usize {
    if points.is_empty() {
        return 0;
    }
    let mut min_x = points[0].x;
    let mut max_x = points[0].x;
    let mut min_y = points[0].y;
    let mut max_y = points[0].y;

    for point in points {
        if point.x < min_x {
            min_x = point.x;
        }
        if point.x > max_x {
            max_x = point.x;
        }
        if point.y < min_y {
            min_y = point.y;
        }
        if point.y > max_y {
            max_y = point.y;
        }
    }

    let bounding_area = ((max_x - min_x + 1) as usize) * ((max_y - min_y + 1) as usize);
    bounding_area - points.len()
}
