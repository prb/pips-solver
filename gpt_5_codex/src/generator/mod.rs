use crate::model::{Board, Piece, Pips, Placement, Point, PolyShape};
use std::collections::HashSet;

pub struct GeneratorConfig {
    pub width: usize,
    pub height: usize,
    pub allowed_shapes: Vec<PolyShape>,
    pub seed: Option<u64>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            width: 6,
            height: 6,
            allowed_shapes: vec![
                PolyShape::Domino,
                PolyShape::I3,
                PolyShape::I4,
                PolyShape::I5,
            ],
            seed: None,
        }
    }
}

pub struct GeneratedPuzzle {
    pub board: Board,
    pub pieces: Vec<Piece>,
    pub placements: Vec<Placement>,
}

struct PlacementSpec {
    shape: PolyShape,
    anchor: Point,
    orientation_index: usize,
    pip_order: Vec<Pips>,
}

pub fn generate(config: GeneratorConfig) -> Result<GeneratedPuzzle, String> {
    if config.width == 0 || config.height == 0 {
        return Err("Board dimensions must be positive.".to_string());
    }
    let mut rng = SimpleRng::new(config.seed, config.width as u64, config.height as u64);

    let mut allowed = if config.allowed_shapes.is_empty() {
        vec![PolyShape::Domino]
    } else {
        config.allowed_shapes.clone()
    };

    // Ensure shapes can tile the area in principle.
    let area = config.width * config.height;
    let gcd = allowed
        .iter()
        .map(|shape| shape.cell_count())
        .fold(0usize, gcd_usize);
    if gcd == 0 || area % gcd != 0 {
        return Err("Board area is incompatible with available shapes.".to_string());
    }

    let mut board_points = HashSet::new();
    for y in 0..config.height as u32 {
        for x in 0..config.width as u32 {
            board_points.insert(Point::new(x, y));
        }
    }

    let mut available = board_points.clone();

    let mut placements = Vec::new();
    if !tile_board(
        &mut available,
        &mut placements,
        allowed.as_mut_slice(),
        &mut rng,
    ) {
        return Err("Failed to tile board with the selected shapes.".to_string());
    }

    let mut pieces = Vec::new();
    let mut rendered = Vec::new();
    for spec in placements.into_iter() {
        let piece = build_piece(spec.shape, &spec.pip_order)?;
        rendered.push(Placement::new(
            piece.clone(),
            spec.anchor,
            spec.orientation_index,
            spec.pip_order.clone(),
        ));
        pieces.push(piece);
    }

    let board = Board::new(board_points);
    Ok(GeneratedPuzzle {
        board,
        pieces,
        placements: rendered,
    })
}

fn tile_board(
    available: &mut HashSet<Point>,
    placements: &mut Vec<PlacementSpec>,
    shapes: &mut [PolyShape],
    rng: &mut SimpleRng,
) -> bool {
    if available.is_empty() {
        return true;
    }

    let mut cells: Vec<Point> = available.iter().copied().collect();
    cells.sort_by_key(|point| (point.y, point.x));
    let pivot = cells[0];

    let mut shape_order: Vec<PolyShape> = shapes.to_vec();
    rng.shuffle(&mut shape_order);
    for shape in shape_order {
        let orientations = shape.orientations();
        for (orientation_index, offsets) in orientations.iter().enumerate() {
            let mut anchors = Vec::new();
            for &(dx, dy) in offsets {
                if let Some(anchor) = anchor_for_offset(pivot, dx, dy) {
                    if !anchors.contains(&anchor) {
                        anchors.push(anchor);
                    }
                }
            }
            rng.shuffle(&mut anchors);
            for anchor in anchors {
                if let Some(cells) = placement_cells(anchor, offsets, available) {
                    let pip_order = random_pips(shape.cell_count(), rng);
                    placements.push(PlacementSpec {
                        shape,
                        anchor,
                        orientation_index,
                        pip_order: pip_order.clone(),
                    });
                    for cell in &cells {
                        available.remove(cell);
                    }
                    if tile_board(available, placements, shapes, rng) {
                        return true;
                    }
                    for cell in cells {
                        available.insert(cell);
                    }
                    placements.pop();
                }
            }
        }
    }
    false
}

fn anchor_for_offset(pivot: Point, dx: i32, dy: i32) -> Option<Point> {
    let anchor_x = pivot.x as i32 - dx;
    let anchor_y = pivot.y as i32 - dy;
    if anchor_x < 0 || anchor_y < 0 {
        return None;
    }
    Some(Point::new(anchor_x as u32, anchor_y as u32))
}

fn placement_cells(
    anchor: Point,
    offsets: &[(i32, i32)],
    available: &HashSet<Point>,
) -> Option<Vec<Point>> {
    let mut cells = Vec::new();
    for &(dx, dy) in offsets {
        let x = anchor.x as i32 + dx;
        let y = anchor.y as i32 + dy;
        if x < 0 || y < 0 {
            return None;
        }
        let point = Point::new(x as u32, y as u32);
        if !available.contains(&point) {
            return None;
        }
        cells.push(point);
    }
    Some(cells)
}

fn random_pips(count: usize, rng: &mut SimpleRng) -> Vec<Pips> {
    (0..count)
        .map(|_| {
            let value = rng.gen_range_inclusive(Pips::MIN, Pips::MAX);
            Pips::new(value).unwrap()
        })
        .collect()
}

fn build_piece(shape: PolyShape, pip_order: &[Pips]) -> Result<Piece, String> {
    if shape == PolyShape::Domino {
        Ok(Piece::domino(pip_order[0], pip_order[1]))
    } else {
        Piece::new(shape, pip_order.to_vec())
    }
}

fn gcd_usize(a: usize, b: usize) -> usize {
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    let mut x = a;
    let mut y = b;
    while y != 0 {
        let temp = y;
        y = x % y;
        x = temp;
    }
    x
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: Option<u64>, width: u64, height: u64) -> Self {
        let mut state = seed.unwrap_or(0x9e37_79b9_7f4a_7c15 ^ width.wrapping_shl(16) ^ height);
        if state == 0 {
            state = 0xfeed_c0de_dead_beef;
        }
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        const A: u64 = 6364136223846793005;
        const C: u64 = 1;
        self.state = self.state.wrapping_mul(A).wrapping_add(C);
        self.state
    }

    fn gen_range_inclusive(&mut self, min: u8, max: u8) -> u8 {
        let span = (max - min + 1) as u64;
        let value = self.next_u64() % span;
        min + value as u8
    }

    fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            slice.swap(i, j);
        }
    }
}
