use crate::model;

/// Render the final board with solver placements overlayed.
pub fn render_board(game: &model::Game, placements: &[model::Placement]) -> Vec<String> {
    let points = game.board.points();
    if points.is_empty() {
        return Vec::new();
    }

    let min_x = points.iter().map(|p| p.x).min().unwrap();
    let max_x = points.iter().map(|p| p.x).max().unwrap();
    let min_y = points.iter().map(|p| p.y).min().unwrap();
    let max_y = points.iter().map(|p| p.y).max().unwrap();

    let width = (max_x - min_x + 1) as usize;
    let height = (max_y - min_y + 1) as usize;

    let mut grid = vec![vec![' '; width]; height];

    for point in points {
        let row = (point.y - min_y) as usize;
        let col = (point.x - min_x) as usize;
        grid[row][col] = '#';
    }

    for placement in placements {
        for assignment in placement.assignments() {
            let point = assignment.point;
            if point.x < min_x || point.x > max_x || point.y < min_y || point.y > max_y {
                continue;
            }
            let row = (point.y - min_y) as usize;
            let col = (point.x - min_x) as usize;
            if let Some(ch) = char::from_digit(assignment.pips.value() as u32, 10) {
                grid[row][col] = ch;
            }
        }
    }

    grid.into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect()
}
