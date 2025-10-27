use crate::model::{Board, Constraint, Game, Piece, Pips, Placement, Point, PolyShape};
use crate::polypips::config::GeneratorConfig;
use crate::polypips::rules::{ConstraintRule, ConstraintSelection, PieceRule};
use crate::util::rng::SimpleRng;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct GeneratedPuzzle {
    pub board: Board,
    pub pieces: Vec<Piece>,
    pub constraints: Vec<Constraint>,
    pub placements: Vec<Placement>,
}

impl GeneratedPuzzle {
    pub fn as_game(&self) -> Game {
        Game::new(
            self.board.clone(),
            self.pieces.clone(),
            self.constraints.clone(),
        )
    }
}

pub fn generate(config: GeneratorConfig) -> Result<GeneratedPuzzle, String> {
    let board_points = config.board.to_hash_set();
    let (width, height) = board_dimensions(&board_points)?;
    let mut rng = SimpleRng::new(config.seed, width as u64, height as u64);

    let piece_specs = tile_board(&board_points, &config.piece_rule, &mut rng)?;

    let constraint_specs = place_constraints(&board_points, &config, &mut rng)?;

    let (constraints, mut board_pips) = assign_constraints(&constraint_specs, &mut rng)?;

    fill_remaining_cells(&board_points, &mut board_pips, &mut rng)?;

    let (pieces, placements) = materialize_pieces(&piece_specs, &board_pips)?;

    let puzzle = GeneratedPuzzle {
        board: config.board,
        pieces,
        constraints,
        placements,
    };
    Ok(puzzle)
}

fn board_dimensions(points: &HashSet<Point>) -> Result<(u32, u32), String> {
    if points.is_empty() {
        return Err("Board must contain at least one point.".to_string());
    }
    let min_x = points.iter().map(|p| p.x).min().unwrap();
    let max_x = points.iter().map(|p| p.x).max().unwrap();
    let min_y = points.iter().map(|p| p.y).min().unwrap();
    let max_y = points.iter().map(|p| p.y).max().unwrap();
    Ok((max_x - min_x + 1, max_y - min_y + 1))
}

#[derive(Clone)]
struct PlacementSpec {
    shape: PolyShape,
    anchor: Point,
    orientation_index: usize,
}

#[derive(Clone)]
struct ShapeRequirement {
    options: Vec<PolyShape>,
}

impl ShapeRequirement {
    fn single(shape: PolyShape) -> Self {
        Self {
            options: vec![shape],
        }
    }

    fn multiple(options: &[PolyShape]) -> Self {
        Self {
            options: options.to_vec(),
        }
    }

    fn cell_count(&self) -> usize {
        self.options
            .first()
            .map(|shape| shape.cell_count())
            .unwrap_or(0)
    }
}

fn tile_board(
    board_points: &HashSet<Point>,
    rule: &PieceRule,
    rng: &mut SimpleRng,
) -> Result<Vec<PlacementSpec>, String> {
    match rule {
        PieceRule::Unlimited(shapes) => tile_unlimited(board_points, shapes, rng),
        PieceRule::Exact(shapes) => {
            let requirements: Vec<ShapeRequirement> = shapes
                .iter()
                .map(|shape| ShapeRequirement::single(*shape))
                .collect();
            tile_exact(board_points, requirements, rng)
        }
        PieceRule::ExactPentominoSet => {
            let board_area = board_points.len();
            if board_area != 60 {
                return Err(
                    "12x5 pentomino set requires a board area of exactly 60 cells.".to_string(),
                );
            }
            let requirements = build_pentomino_requirements();
            tile_exact(board_points, requirements, rng)
        }
    }
}

fn tile_unlimited(
    board_points: &HashSet<Point>,
    shapes: &[PolyShape],
    rng: &mut SimpleRng,
) -> Result<Vec<PlacementSpec>, String> {
    if shapes.is_empty() {
        return Err("Pieces rule resolved to an empty shape set.".to_string());
    }
    let gcd = shapes
        .iter()
        .map(|shape| shape.cell_count())
        .fold(0usize, gcd_usize);
    if gcd == 0 || board_points.len() % gcd != 0 {
        return Err("Board area is incompatible with the chosen piece shapes.".to_string());
    }

    let mut available = board_points.clone();
    let mut placements = Vec::new();
    if backtrack_unlimited(&mut available, &mut placements, shapes, rng) {
        Ok(placements)
    } else {
        Err("Failed to tile the board with the allowed shapes.".to_string())
    }
}

fn tile_exact(
    board_points: &HashSet<Point>,
    requirements: Vec<ShapeRequirement>,
    rng: &mut SimpleRng,
) -> Result<Vec<PlacementSpec>, String> {
    if requirements.is_empty() {
        return Err("Exact piece rule requires at least one shape.".to_string());
    }
    let total: usize = requirements.iter().map(|req| req.cell_count()).sum();
    if total != board_points.len() {
        return Err("Exact piece rule total area does not match board area.".to_string());
    }

    let mut ordered = requirements;
    rng.shuffle(&mut ordered);
    let mut available = board_points.clone();
    let mut placements = Vec::new();
    if backtrack_exact(&mut available, &mut placements, &mut ordered, rng) {
        Ok(placements)
    } else {
        Err("Failed to tile the board with the requested exact shapes.".to_string())
    }
}

fn build_pentomino_requirements() -> Vec<ShapeRequirement> {
    vec![
        ShapeRequirement::multiple(&[PolyShape::PentFPlus, PolyShape::PentFMinus]),
        ShapeRequirement::single(PolyShape::PentI),
        ShapeRequirement::multiple(&[PolyShape::PentLPlus, PolyShape::PentLMinus]),
        ShapeRequirement::multiple(&[PolyShape::PentPPlus, PolyShape::PentPMinus]),
        ShapeRequirement::multiple(&[PolyShape::PentNPlus, PolyShape::PentNMinus]),
        ShapeRequirement::single(PolyShape::PentT),
        ShapeRequirement::single(PolyShape::PentU),
        ShapeRequirement::single(PolyShape::PentV),
        ShapeRequirement::single(PolyShape::PentW),
        ShapeRequirement::single(PolyShape::PentX),
        ShapeRequirement::multiple(&[PolyShape::PentYPlus, PolyShape::PentYMinus]),
        ShapeRequirement::multiple(&[PolyShape::PentZPlus, PolyShape::PentZMinus]),
    ]
}

fn backtrack_unlimited(
    available: &mut HashSet<Point>,
    placements: &mut Vec<PlacementSpec>,
    shapes: &[PolyShape],
    rng: &mut SimpleRng,
) -> bool {
    if available.is_empty() {
        return true;
    }
    let pivot = pick_pivot(available);

    let mut shape_order: Vec<PolyShape> = shapes.to_vec();
    rng.shuffle(&mut shape_order);
    for shape in shape_order {
        let orientations = shape.orientations();
        let mut orientation_indices: Vec<usize> = (0..orientations.len()).collect();
        rng.shuffle(&mut orientation_indices);
        for orientation_index in orientation_indices {
            let offsets = &orientations[orientation_index];
            let mut anchors = anchors_for_pivot(pivot, offsets);
            rng.shuffle(&mut anchors);
            for anchor in anchors {
                if let Some(cells) = placement_cells_available(anchor, offsets, available) {
                    placements.push(PlacementSpec {
                        shape,
                        anchor,
                        orientation_index,
                    });
                    for cell in &cells {
                        available.remove(cell);
                    }
                    if backtrack_unlimited(available, placements, shapes, rng) {
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

fn backtrack_exact(
    available: &mut HashSet<Point>,
    placements: &mut Vec<PlacementSpec>,
    requirements: &mut Vec<ShapeRequirement>,
    rng: &mut SimpleRng,
) -> bool {
    if requirements.is_empty() {
        return available.is_empty();
    }
    if available.is_empty() {
        return false;
    }

    let pivot = pick_pivot(available);
    let mut requirement_indices: Vec<usize> = (0..requirements.len()).collect();
    rng.shuffle(&mut requirement_indices);

    for req_idx in requirement_indices {
        let requirement = requirements.remove(req_idx);
        let mut option_indices: Vec<usize> = (0..requirement.options.len()).collect();
        rng.shuffle(&mut option_indices);

        for option_idx in option_indices {
            let shape = requirement.options[option_idx];
            let orientations = shape.orientations();
            let mut orientation_indices: Vec<usize> = (0..orientations.len()).collect();
            rng.shuffle(&mut orientation_indices);

            for orientation_index in orientation_indices {
                let offsets = &orientations[orientation_index];
                let mut anchors = anchors_for_pivot(pivot, offsets);
                rng.shuffle(&mut anchors);
                for anchor in anchors {
                    if let Some(cells) = placement_cells_available(anchor, offsets, available) {
                        placements.push(PlacementSpec {
                            shape,
                            anchor,
                            orientation_index,
                        });
                        for cell in &cells {
                            available.remove(cell);
                        }

                        if backtrack_exact(available, placements, requirements, rng) {
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

        requirements.insert(req_idx, requirement);
    }

    false
}

fn pick_pivot(available: &HashSet<Point>) -> Point {
    available
        .iter()
        .copied()
        .min_by_key(|point| (point.y, point.x))
        .expect("available set not empty")
}

fn anchors_for_pivot(pivot: Point, offsets: &[(i32, i32)]) -> Vec<Point> {
    let mut anchors = Vec::new();
    for &(dx, dy) in offsets {
        if let Some(anchor) = anchor_for_offset(pivot, dx, dy) {
            if !anchors.contains(&anchor) {
                anchors.push(anchor);
            }
        }
    }
    anchors
}

fn anchor_for_offset(pivot: Point, dx: i32, dy: i32) -> Option<Point> {
    let anchor_x = pivot.x as i32 - dx;
    let anchor_y = pivot.y as i32 - dy;
    if anchor_x < 0 || anchor_y < 0 {
        return None;
    }
    Some(Point::new(anchor_x as u32, anchor_y as u32))
}

fn placement_cells_available(
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

#[derive(Clone)]
struct ConstraintSpec {
    shape: PolyShape,
    anchor: Point,
    orientation_index: usize,
}

fn place_constraints(
    board_points: &HashSet<Point>,
    config: &GeneratorConfig,
    rng: &mut SimpleRng,
) -> Result<Vec<ConstraintSpec>, String> {
    let shapes = match &config.constraint_rule {
        ConstraintRule::None => return Ok(Vec::new()),
        ConstraintRule::Allowed(shapes) => shapes.clone(),
    };
    if shapes.is_empty() || config.coverage <= 0.0 {
        return Ok(Vec::new());
    }

    let target_cells =
        ((config.coverage * board_points.len() as f64).round() as usize).min(board_points.len());
    if target_cells == 0 {
        return Ok(Vec::new());
    }

    let mut occupied = HashSet::new();
    let mut placements = Vec::new();
    let max_attempts = board_points.len() * 50;
    let mut attempts = 0usize;

    while occupied.len() < target_cells && attempts < max_attempts {
        attempts += 1;
        if let Some(spec) =
            find_constraint_placement(board_points, &occupied, &shapes, config.selection, rng)
        {
            let offsets = spec.shape.orientations()[spec.orientation_index].clone();
            let cells: Vec<Point> = offsets
                .iter()
                .map(|&(dx, dy)| {
                    Point::new(
                        (spec.anchor.x as i32 + dx) as u32,
                        (spec.anchor.y as i32 + dy) as u32,
                    )
                })
                .collect();
            let mut new_cells = 0usize;
            for cell in &cells {
                if !occupied.contains(cell) {
                    new_cells += 1;
                }
            }
            if new_cells == 0 {
                continue;
            }
            for cell in cells {
                occupied.insert(cell);
            }
            placements.push(spec);
        } else {
            break;
        }
    }

    if occupied.len() < target_cells {
        return Err(
            "Unable to achieve the requested constraint coverage with the given rules.".to_string(),
        );
    }

    Ok(placements)
}

fn find_constraint_placement(
    board_points: &HashSet<Point>,
    occupied: &HashSet<Point>,
    shapes: &[PolyShape],
    selection: ConstraintSelection,
    rng: &mut SimpleRng,
) -> Option<ConstraintSpec> {
    let mut attempts = 0usize;
    let available_points: Vec<Point> = board_points.difference(occupied).copied().collect();
    if available_points.is_empty() {
        return None;
    }

    let mut shapes_by_size: HashMap<usize, Vec<PolyShape>> = HashMap::new();
    if matches!(selection, ConstraintSelection::UniformSize) {
        for &shape in shapes {
            shapes_by_size
                .entry(shape.cell_count())
                .or_insert_with(Vec::new)
                .push(shape);
        }
    }

    let mut shuffled_points = available_points.clone();
    rng.shuffle(&mut shuffled_points);

    let max_attempts = shapes.len() * board_points.len();
    while attempts < max_attempts {
        attempts += 1;
        let shape = match selection {
            ConstraintSelection::UniformAll => {
                let idx = rng.gen_range_usize(0, shapes.len() - 1);
                shapes[idx]
            }
            ConstraintSelection::UniformSize => {
                let mut sizes: Vec<usize> = shapes_by_size.keys().copied().collect();
                if sizes.is_empty() {
                    return None;
                }
                rng.shuffle(&mut sizes);
                let chosen_size = sizes[0];
                let pool = shapes_by_size.get(&chosen_size)?;
                let idx = rng.gen_range_usize(0, pool.len() - 1);
                pool[idx]
            }
        };

        let orientations = shape.orientations();
        let mut orientation_indices: Vec<usize> = (0..orientations.len()).collect();
        rng.shuffle(&mut orientation_indices);

        for orientation_index in orientation_indices {
            let offsets = &orientations[orientation_index];
            for pivot in &shuffled_points {
                let mut anchors = anchors_for_pivot(*pivot, offsets);
                rng.shuffle(&mut anchors);
                for anchor in anchors {
                    if constraint_cells(board_points, occupied, anchor, offsets).is_some() {
                        return Some(ConstraintSpec {
                            shape,
                            anchor,
                            orientation_index,
                        });
                    }
                }
            }
        }
    }
    None
}

fn constraint_cells(
    board_points: &HashSet<Point>,
    occupied: &HashSet<Point>,
    anchor: Point,
    offsets: &[(i32, i32)],
) -> Option<Vec<Point>> {
    let mut cells = Vec::new();
    for &(dx, dy) in offsets {
        let x = anchor.x as i32 + dx;
        let y = anchor.y as i32 + dy;
        if x < 0 || y < 0 {
            return None;
        }
        let point = Point::new(x as u32, y as u32);
        if !board_points.contains(&point) || occupied.contains(&point) {
            return None;
        }
        cells.push(point);
    }
    Some(cells)
}

fn assign_constraints(
    specs: &[ConstraintSpec],
    rng: &mut SimpleRng,
) -> Result<(Vec<Constraint>, HashMap<Point, Pips>), String> {
    let mut constraints = Vec::new();
    let mut board_pips = HashMap::new();

    for spec in specs {
        let offsets = spec.shape.orientations()[spec.orientation_index].clone();
        let mut points = Vec::new();
        for &(dx, dy) in &offsets {
            let x = (spec.anchor.x as i32 + dx) as u32;
            let y = (spec.anchor.y as i32 + dy) as u32;
            points.push(Point::new(x, y));
        }
        let (constraint, assignments) = generate_constraint(points, rng)?;
        for (point, pip) in &assignments {
            board_pips.insert(*point, *pip);
        }
        constraints.push(constraint);
    }

    Ok((constraints, board_pips))
}

fn generate_constraint(
    points: Vec<Point>,
    rng: &mut SimpleRng,
) -> Result<(Constraint, Vec<(Point, Pips)>), String> {
    let mut choices = vec![
        ConstraintKind::AllSame,
        ConstraintKind::Exactly,
        ConstraintKind::LessThan,
        ConstraintKind::MoreThan,
    ];
    if points.len() > 1 && points.len() <= (Pips::MAX as usize + 1) {
        choices.push(ConstraintKind::AllDifferent);
    }
    let idx = rng.gen_range_usize(0, choices.len() - 1);
    let kind = choices[idx];
    build_constraint(points, kind, rng)
}

#[derive(Clone, Copy)]
enum ConstraintKind {
    AllSame,
    AllDifferent,
    Exactly,
    LessThan,
    MoreThan,
}

fn build_constraint(
    points: Vec<Point>,
    kind: ConstraintKind,
    rng: &mut SimpleRng,
) -> Result<(Constraint, Vec<(Point, Pips)>), String> {
    let points_set: Arc<HashSet<Point>> = Arc::new(points.iter().copied().collect());
    match kind {
        ConstraintKind::AllSame => {
            let value = random_pip(rng);
            let assignments: Vec<(Point, Pips)> = points.iter().map(|p| (*p, value)).collect();
            let constraint = Constraint::AllSame {
                expected: Some(value),
                points: Arc::clone(&points_set),
            };
            Ok((constraint, assignments))
        }
        ConstraintKind::AllDifferent => {
            let mut values: Vec<Pips> = (Pips::MIN..=Pips::MAX)
                .map(|v| Pips::new(v).unwrap())
                .collect();
            rng.shuffle(&mut values);
            let assignments: Vec<(Point, Pips)> = points
                .iter()
                .zip(values.into_iter())
                .take(points.len())
                .map(|(p, pip)| (*p, pip))
                .collect();
            let constraint = Constraint::AllDifferent {
                excluded: Arc::new(HashSet::new()),
                points: Arc::clone(&points_set),
            };
            Ok((constraint, assignments))
        }
        ConstraintKind::Exactly => {
            let assignments = random_assignment(&points, rng);
            let sum: u32 = assignments.iter().map(|(_, pip)| pip.value() as u32).sum();
            let constraint = Constraint::Exactly {
                target: sum,
                points: Arc::clone(&points_set),
            };
            Ok((constraint, assignments))
        }
        ConstraintKind::LessThan => {
            let max_sum = (points.len() as u32) * (Pips::MAX as u32);
            loop {
                let sample = random_assignment(&points, rng);
                let sum: u32 = sample.iter().map(|(_, pip)| pip.value() as u32).sum();
                if sum < max_sum {
                    let remaining = max_sum - (sum + 1);
                    let offset = if remaining == 0 {
                        0
                    } else {
                        rng.gen_range_usize(0, remaining as usize) as u32
                    };
                    let target = sum + 1 + offset;
                    let constraint = Constraint::LessThan {
                        target,
                        points: Arc::clone(&points_set),
                    };
                    return Ok((constraint, sample));
                }
            }
        }
        ConstraintKind::MoreThan => loop {
            let sample = random_assignment(&points, rng);
            let sum: u32 = sample.iter().map(|(_, pip)| pip.value() as u32).sum();
            if sum > 0 {
                let target = rng.gen_range_usize(0, (sum - 1) as usize) as u32;
                let constraint = Constraint::MoreThan {
                    target,
                    points: Arc::clone(&points_set),
                };
                return Ok((constraint, sample));
            }
        },
    }
}

fn random_assignment(points: &[Point], rng: &mut SimpleRng) -> Vec<(Point, Pips)> {
    points
        .iter()
        .map(|point| (*point, random_pip(rng)))
        .collect()
}

fn random_pip(rng: &mut SimpleRng) -> Pips {
    let value = rng.gen_range_inclusive(Pips::MIN, Pips::MAX);
    Pips::new(value).expect("range produces valid pip")
}

fn fill_remaining_cells(
    board_points: &HashSet<Point>,
    board_pips: &mut HashMap<Point, Pips>,
    rng: &mut SimpleRng,
) -> Result<(), String> {
    for point in board_points {
        board_pips.entry(*point).or_insert_with(|| random_pip(rng));
    }
    Ok(())
}

fn materialize_pieces(
    specs: &[PlacementSpec],
    board_pips: &HashMap<Point, Pips>,
) -> Result<(Vec<Piece>, Vec<Placement>), String> {
    let mut pieces = Vec::new();
    let mut placements = Vec::new();

    for spec in specs {
        let offsets = spec.shape.orientations()[spec.orientation_index].clone();
        let mut pip_order = Vec::new();
        for &(dx, dy) in &offsets {
            let point = Point::new(
                (spec.anchor.x as i32 + dx) as u32,
                (spec.anchor.y as i32 + dy) as u32,
            );
            let pip = board_pips
                .get(&point)
                .ok_or_else(|| format!("Missing pip assignment for point {}", point))?;
            pip_order.push(*pip);
        }
        let piece = Piece::new(spec.shape, pip_order.clone())
            .map_err(|err| format!("Failed to create piece: {}", err))?;
        placements.push(Placement::new(
            piece.clone(),
            spec.anchor,
            spec.orientation_index,
            pip_order,
        ));
        pieces.push(piece);
    }

    Ok((pieces, placements))
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
