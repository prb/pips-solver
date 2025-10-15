// pips-solver/src/main.rs

use anyhow::{anyhow, Context, Result};
use std::env;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// =============================================================================
// Data Model Module
// =============================================================================
pub mod data_model {
    use std::collections::HashSet;
    use std::hash::Hash;

    // Pips
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
    pub struct Pips(u8);

    impl Pips {
        pub fn new(value: u8) -> Option<Self> {
            if value <= 6 {
                Some(Self(value))
            } else {
                None
            }
        }
        pub fn get(&self) -> u8 {
            self.0
        }
    }

    // Point
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
    pub struct Point {
        pub x: usize,
        pub y: usize,
    }

    impl Point {
        pub fn new(x: usize, y: usize) -> Self {
            Self { x, y }
        }
    }

    // Piece
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Piece {
        pub v1: Pips,
        pub v2: Pips,
    }

    impl Piece {
        pub fn new(pips1: Pips, pips2: Pips) -> Self {
            if pips1 <= pips2 {
                Self {
                    v1: pips1,
                    v2: pips2,
                }
            } else {
                Self {
                    v1: pips2,
                    v2: pips1,
                }
            }
        }
        pub fn is_doubleton(&self) -> bool {
            self.v1 == self.v2
        }
    }

    // Board
    #[derive(Debug, PartialEq, Eq, Clone, Default)]
    pub struct Board {
        pub points: HashSet<Point>,
    }

    impl Board {
        pub fn new() -> Self {
            Self::default()
        }
        pub fn from_points(points: HashSet<Point>) -> Self {
            Self { points }
        }
        pub fn get_next_point(&self) -> Option<Point> {
            self.points.iter().min().copied()
        }
        pub fn is_valid(&self) -> bool {
            if self.points.is_empty() {
                return true;
            }
            for point in &self.points {
                let has_neighbor = self.points.contains(&Point::new(point.x + 1, point.y))
                    || (point.x > 0 && self.points.contains(&Point::new(point.x - 1, point.y)))
                    || self.points.contains(&Point::new(point.x, point.y + 1))
                    || (point.y > 0 && self.points.contains(&Point::new(point.x, point.y - 1)));
                if !has_neighbor {
                    return false;
                }
            }
            true
        }
    }

    // Assignment
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct Assignment {
        pub pips: Pips,
        pub point: Point,
    }

    impl Assignment {
        pub fn new(pips: Pips, point: Point) -> Self {
            Self { pips, point }
        }
    }

    // Direction
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Direction {
        North,
        South,
        East,
        West,
    }

    // Constraint
    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum Constraint {
        AllSame(Option<Pips>, HashSet<Point>),
        AllDifferent(HashSet<Pips>, HashSet<Point>),
        LessThan(u32, HashSet<Point>),
        Exactly(u32, HashSet<Point>),
        MoreThan(u32, HashSet<Point>),
    }

    // Placement
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Placement {
        pub piece: Piece,
        pub point: Point,
        pub direction: Direction,
    }

    impl Placement {
        pub fn new(piece: Piece, point: Point, direction: Direction) -> Self {
            Self {
                piece,
                point,
                direction,
            }
        }
        pub fn assignments(&self) -> [Assignment; 2] {
            let (p1, p2) = (self.piece.v1, self.piece.v2);
            let (x, y) = (self.point.x, self.point.y);
            match self.direction {
                Direction::North => [
                    Assignment::new(p2, Point::new(x, y)),
                    Assignment::new(p1, Point::new(x, y + 1)),
                ],
                Direction::South => [
                    Assignment::new(p1, Point::new(x, y)),
                    Assignment::new(p2, Point::new(x, y + 1)),
                ],
                Direction::West => [
                    Assignment::new(p2, Point::new(x, y)),
                    Assignment::new(p1, Point::new(x + 1, y)),
                ],
                Direction::East => [
                    Assignment::new(p1, Point::new(x, y)),
                    Assignment::new(p2, Point::new(x + 1, y)),
                ],
            }
        }
    }

    // Game
    #[derive(Debug, Clone)]
    pub struct Game {
        pub board: Board,
        pub pieces: Vec<Piece>,
        pub constraints: Vec<Constraint>,
    }

    impl Game {
        pub fn new(board: Board, pieces: Vec<Piece>, constraints: Vec<Constraint>) -> Self {
            Self {
                board,
                pieces,
                constraints,
            }
        }
        pub fn is_won(&self) -> bool {
            self.board.points.is_empty() && self.pieces.is_empty()
        }
    }
}

// =============================================================================
// Game Loader Module
// =============================================================================
pub mod game_loader {
    use crate::data_model::{Constraint, Game};
    use anyhow::{anyhow, Context, Result};
    use std::collections::HashSet;
    use std::fs;

    // Parser Submodule
    pub mod parser {
        use crate::data_model::{Board, Constraint, Game, Piece, Pips, Point};
        use nom::{
            branch::alt,
            bytes::complete::{tag, take_while1},
            character::complete::{
                char, line_ending, multispace0, multispace1, not_line_ending, one_of,
            },
            combinator::{map, map_res, opt},
            multi::{many0, separated_list0, separated_list1},
            sequence::{delimited, preceded, separated_pair, terminated},
            IResult, Parser,
        };
        use std::collections::HashSet;

        type ParseResult<'a, O> = IResult<&'a str, O, nom::error::Error<&'a str>>;

        fn from_dec(input: &str) -> Result<usize, std::num::ParseIntError> {
            usize::from_str_radix(input, 10)
        }

        fn from_dec_u32(input: &str) -> Result<u32, std::num::ParseIntError> {
            u32::from_str_radix(input, 10)
        }

        fn is_dec_digit(c: char) -> bool {
            c.is_ascii_digit()
        }

        fn parse_pips(input: &str) -> ParseResult<'_, Pips> {
            let (input, c) = one_of("0123456")(input)?;
            let pips = Pips::new(c.to_digit(10).unwrap() as u8).unwrap();
            Ok((input, pips))
        }

        fn parse_point(input: &str) -> ParseResult<'_, Point> {
            map(
                delimited(
                    char('('),
                    separated_pair(
                        map_res(take_while1(is_dec_digit), from_dec),
                        delimited(multispace0, char(','), multispace0),
                        map_res(take_while1(is_dec_digit), from_dec),
                    ),
                    char(')'),
                ),
                |(x, y)| Point::new(x, y),
            )
            .parse(input)
        }

        fn parse_points_set(input: &str) -> ParseResult<'_, HashSet<Point>> {
            map(
                delimited(
                    char('{'),
                    separated_list0(delimited(multispace0, char(','), multispace0), parse_point),
                    char('}'),
                ),
                |points| points.into_iter().collect(),
            )
            .parse(input)
        }

        fn is_board_char(c: char) -> bool {
            c == '#' || c == ' '
        }

        fn parse_board_line(input: &str) -> ParseResult<'_, &str> {
            let (input, line) = take_while1(is_board_char)(input)?;
            let (input, _) = line_ending(input)?;
            Ok((input, line))
        }

        fn parse_board(input: &str) -> ParseResult<'_, Board> {
            let (input, lines) = many0(parse_board_line).parse(input)?;
            let mut points = HashSet::new();
            for (y, line) in lines.iter().enumerate() {
                for (x, c) in line.chars().enumerate() {
                    if c == '#' {
                        points.insert(Point::new(x, y));
                    }
                }
            }
            Ok((input, Board::from_points(points)))
        }

        fn parse_piece(input: &str) -> ParseResult<'_, Piece> {
            map((parse_pips, parse_pips), |(p1, p2)| Piece::new(p1, p2)).parse(input)
        }

        fn parse_pieces_list(input: &str) -> ParseResult<'_, Vec<Piece>> {
            separated_list1(delimited(multispace0, char(','), multispace0), parse_piece)
                .parse(input)
        }

        fn parse_all_same(input: &str) -> ParseResult<'_, Constraint> {
            map(
                (
                    tag("AllSame"),
                    multispace1,
                    alt((map(parse_pips, Some), map(tag("None"), |_| None))),
                    multispace1,
                    parse_points_set,
                ),
                |(_, _, pips, _, points)| Constraint::AllSame(pips, points),
            )
            .parse(input)
        }

        fn parse_all_different(input: &str) -> ParseResult<'_, Constraint> {
            map(
                (
                    tag("AllDifferent"),
                    multispace1,
                    delimited(
                        char('{'),
                        map(
                            separated_list0(
                                delimited(multispace0, char(','), multispace0),
                                parse_pips,
                            ),
                            |pips| pips.into_iter().collect(),
                        ),
                        char('}'),
                    ),
                    multispace1,
                    parse_points_set,
                ),
                |(_, _, pips, _, points)| Constraint::AllDifferent(pips, points),
            )
            .parse(input)
        }

        fn parse_sum_constraint(input: &str) -> ParseResult<'_, Constraint> {
            let (input, (constraint_type, _, target, _, points)) = (
                alt((tag("Exactly"), tag("LessThan"), tag("MoreThan"))),
                multispace1,
                map_res(take_while1(is_dec_digit), from_dec_u32),
                multispace1,
                parse_points_set,
            )
                .parse(input)?;
            let constraint = match constraint_type {
                "Exactly" => Constraint::Exactly(target, points),
                "LessThan" => Constraint::LessThan(target, points),
                "MoreThan" => Constraint::MoreThan(target, points),
                _ => unreachable!(),
            };
            Ok((input, constraint))
        }

        fn parse_constraint(input: &str) -> ParseResult<'_, Constraint> {
            alt((parse_all_same, parse_all_different, parse_sum_constraint)).parse(input)
        }

        fn parse_comment_line(input: &str) -> ParseResult<'_, Option<Constraint>> {
            map(preceded((multispace0, tag("//")), not_line_ending), |_| {
                None
            })
            .parse(input)
        }

        fn parse_constraint_entry(input: &str) -> ParseResult<'_, Option<Constraint>> {
            alt((map(parse_constraint, Some), parse_comment_line)).parse(input)
        }

        fn parse_constraints_list(input: &str) -> ParseResult<'_, Vec<Constraint>> {
            map(
                separated_list0(multispace1, parse_constraint_entry),
                |opts| opts.into_iter().flatten().collect(),
            )
            .parse(input)
        }

        fn skip_header(input: &str) -> ParseResult<'_, ()> {
            let line_content = alt((
                preceded(tag("//"), not_line_ending),
                take_while1(|c: char| c.is_whitespace() && c != '\n' && c != '\r'),
            ));
            map(many0(terminated(line_content, line_ending)), |_| ()).parse(input)
        }

        pub fn parse_game(input: &str) -> ParseResult<'_, Game> {
            let (input, _) = opt(skip_header).parse(input)?;
            let (input, _) = terminated(tag("board:"), line_ending).parse(input)?;
            let (input, board) = parse_board(input)?;
            let (input, _) = multispace0(input)?;
            let (input, _) = terminated(tag("pieces:"), multispace1).parse(input)?;
            let (input, pieces) = terminated(parse_pieces_list, multispace1).parse(input)?;
            let (input, constraints) = opt(preceded(
                terminated(tag("constraints:"), multispace1),
                parse_constraints_list,
            ))
            .parse(input)?;
            let (input, _) = multispace0(input)?;
            Ok((
                input,
                Game::new(board, pieces, constraints.unwrap_or_default()),
            ))
        }
    }

    fn validate_constraint_consistency(constraints: &[Constraint]) -> Result<()> {
        let mut seen_points = HashSet::new();
        for constraint in constraints {
            let points = match constraint {
                Constraint::AllSame(_, p) => p,
                Constraint::AllDifferent(_, p) => p,
                Constraint::Exactly(_, p) => p,
                Constraint::LessThan(_, p) => p,
                Constraint::MoreThan(_, p) => p,
            };
            for point in points {
                if !seen_points.insert(point) {
                    return Err(anyhow!(
                        "Validation Error: Point ({}, {}) is used in multiple constraints.",
                        point.x,
                        point.y
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn load_game_from_file(path: &str) -> Result<Game> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Could not read file at '{}'", path))?;
        match parser::parse_game(&contents) {
            Ok((remaining_input, game)) => {
                if !remaining_input.trim().is_empty() {
                    return Err(anyhow!(
                        "Parser finished but did not consume all input. Remaining: '{}'",
                        remaining_input
                    ));
                }
                validate_constraint_consistency(&game.constraints)?;
                Ok(game)
            }
            Err(e) => Err(anyhow!("Failed to parse game file: {}", e.to_string())),
        }
    }
}

// =============================================================================
// Solver Module
// =============================================================================
pub mod solver {
    use crate::data_model::{Direction, Game, Placement};

    // Reduction Submodule
    pub mod reduction {
        use crate::data_model::{Assignment, Board, Constraint, Piece, Placement, Point};
        use std::collections::HashSet;

        type ReductionResult = Result<Option<Constraint>, ()>;

        pub fn reduce_a(constraint: &Constraint, assignment: &Assignment) -> ReductionResult {
            let pips = assignment.pips;
            let point = assignment.point;
            match constraint {
                Constraint::AllSame(target_pips, points) => {
                    if !points.contains(&point) {
                        return Ok(Some(constraint.clone()));
                    }
                    if let Some(target) = target_pips {
                        if *target != pips {
                            return Err(());
                        }
                    }
                    let mut new_points = points.clone();
                    new_points.remove(&point);
                    match points.len() {
                        1 => Ok(None),
                        2 => {
                            let target_sum = pips.get() as u32;
                            Ok(Some(Constraint::Exactly(target_sum, new_points)))
                        }
                        _ => Ok(Some(Constraint::AllSame(Some(pips), new_points))),
                    }
                }
                Constraint::AllDifferent(avoid_pips, points) => {
                    if !points.contains(&point) {
                        return Ok(Some(constraint.clone()));
                    }
                    if avoid_pips.contains(&pips) {
                        return Err(());
                    }
                    let mut new_points = points.clone();
                    new_points.remove(&point);
                    if new_points.is_empty() {
                        Ok(None)
                    } else {
                        let mut new_avoid = avoid_pips.clone();
                        new_avoid.insert(pips);
                        Ok(Some(Constraint::AllDifferent(new_avoid, new_points)))
                    }
                }
                Constraint::Exactly(sum, points) => {
                    if !points.contains(&point) {
                        return Ok(Some(constraint.clone()));
                    }
                    let pips_val = pips.get() as u32;
                    if pips_val > *sum {
                        return Err(());
                    }
                    let new_sum = sum - pips_val;
                    let mut new_points = points.clone();
                    new_points.remove(&point);

                    // NEW LOGIC: Check if the new sum is achievable.
                    let max_possible_sum = new_points.len() as u32 * 6;
                    if new_sum > max_possible_sum {
                        return Err(()); // This branch is impossible.
                    }

                    if new_points.is_empty() {
                        if new_sum == 0 {
                            Ok(None)
                        } else {
                            Err(())
                        }
                    } else {
                        Ok(Some(Constraint::Exactly(new_sum, new_points)))
                    }
                }
                Constraint::LessThan(sum, points) => {
                    if !points.contains(&point) {
                        return Ok(Some(constraint.clone()));
                    }
                    let pips_val = pips.get() as u32;
                    if pips_val >= *sum {
                        return Err(());
                    }
                    let new_sum = sum - pips_val;
                    let mut new_points = points.clone();
                    new_points.remove(&point);
                    if new_points.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(Constraint::LessThan(new_sum, new_points)))
                    }
                }
                Constraint::MoreThan(sum, points) => {
                    if !points.contains(&point) {
                        return Ok(Some(constraint.clone()));
                    }
                    let pips_val = pips.get() as u32;
                    let mut new_points = points.clone();
                    new_points.remove(&point);
                    if new_points.is_empty() {
                        if pips_val > *sum {
                            Ok(None)
                        } else {
                            Err(())
                        }
                    } else {
                        let new_sum = sum.saturating_sub(pips_val);
                        Ok(Some(Constraint::MoreThan(new_sum, new_points)))
                    }
                }
            }
        }

        pub fn reduce_p(constraint: &Constraint, placement: &Placement) -> ReductionResult {
            let constraint_points = match constraint {
                Constraint::AllSame(_, p) => p,
                Constraint::AllDifferent(_, p) => p,
                Constraint::Exactly(_, p) => p,
                Constraint::LessThan(_, p) => p,
                Constraint::MoreThan(_, p) => p,
            };

            placement
                .assignments()
                .into_iter()
                .filter(|a| constraint_points.contains(&a.point))
                .try_fold(Some(constraint.clone()), |acc, assignment| match acc {
                    Some(c) => reduce_a(&c, &assignment),
                    None => Ok(None),
                })
        }

        pub fn reduce_cs(
            constraints: &[Constraint],
            placement: &Placement,
        ) -> Option<Vec<Constraint>> {
            let mut next_constraints = Vec::with_capacity(constraints.len());
            for constraint in constraints {
                match reduce_p(constraint, placement) {
                    Ok(Some(next_constraint)) => next_constraints.push(next_constraint),
                    Ok(None) => {}
                    Err(_) => return None,
                }
            }
            Some(next_constraints)
        }

        fn placement_points(placement: &Placement) -> HashSet<Point> {
            placement.assignments().iter().map(|a| a.point).collect()
        }

        pub fn reduce_b(board: &Board, placement: &Placement) -> Option<Board> {
            let p_points = placement_points(placement);
            if p_points.is_subset(&board.points) {
                let new_board_points = board.points.difference(&p_points).cloned().collect();
                let new_board = Board::from_points(new_board_points);
                if new_board.is_valid() {
                    Some(new_board)
                } else {
                    None
                }
            } else {
                None
            }
        }

        pub fn remove_one_piece(pieces: &[Piece], piece_to_remove: &Piece) -> Option<Vec<Piece>> {
            if let Some(index) = pieces.iter().position(|p| p == piece_to_remove) {
                let mut new_pieces = pieces.to_vec();
                new_pieces.remove(index);
                Some(new_pieces)
            } else {
                None
            }
        }
    }

    fn play(game: &Game, placement: &Placement) -> Option<Game> {
        let next_board = reduction::reduce_b(&game.board, placement)?;
        let next_pieces = reduction::remove_one_piece(&game.pieces, &placement.piece)?;
        let next_constraints = reduction::reduce_cs(&game.constraints, placement)?;
        Some(Game::new(next_board, next_pieces, next_constraints))
    }

    pub fn solve_game(game: &Game) -> Option<Vec<Placement>> {
        solve_recursive(game.clone(), Vec::new())
    }

    fn solve_recursive(current_game: Game, path: Vec<Placement>) -> Option<Vec<Placement>> {
        if current_game.is_won() {
            return Some(path);
        }
        let next_point = match current_game.board.get_next_point() {
            Some(point) => point,
            None => return None,
        };
        for piece in &current_game.pieces {
            let directions_to_try = if piece.is_doubleton() {
                vec![Direction::South, Direction::East]
            } else {
                vec![
                    Direction::North,
                    Direction::South,
                    Direction::East,
                    Direction::West,
                ]
            };
            for direction in directions_to_try {
                let placement = Placement::new(*piece, next_point, direction);
                if let Some(next_game) = play(&current_game, &placement) {
                    let mut new_path = path.clone();
                    new_path.push(placement);
                    if let Some(solution) = solve_recursive(next_game, new_path) {
                        return Some(solution);
                    }
                }
            }
        }
        None
    }
}

// =============================================================================
// Main Function
// =============================================================================
fn main() -> Result<()> {
    println!("Pips Solver");

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!(
            "Usage: {} <path/to/game_file.txt>",
            args.get(0).unwrap_or(&"pips-solver".to_string())
        );
        return Err(anyhow!("Invalid number of arguments"));
    }

    let file_path = &args[1];
    println!("Loading game from: {}", file_path);

    let game = game_loader::load_game_from_file(file_path)
        .with_context(|| format!("Failed to load game from file: '{}'", file_path))?;
    println!("Game loaded successfully!");

    println!("Solving...");
    let start_time = std::time::Instant::now();

    // Create a channel for communication
    let (tx, rx) = mpsc::channel();
    let game_clone = game.clone();

    // Spawn the solver in a new thread
    thread::spawn(move || {
        let solution = solver::solve_game(&game_clone);
        // The send can fail if the receiver has been dropped, which is fine.
        let _ = tx.send(solution);
    });

    let timeout = Duration::from_secs(60);
    let result = rx.recv_timeout(timeout);
    let duration = start_time.elapsed();

    match result {
        Ok(solution) => match solution {
            Some(placements) => {
                println!("\nSolution found in {:.2?}!", duration);
                println!("Placements:");
                for p in placements {
                    println!(
                        "  - Piece ({}, {}) at ({}, {}) -> {:?}",
                        p.piece.v1.get(),
                        p.piece.v2.get(),
                        p.point.x,
                        p.point.y,
                        p.direction
                    );
                }
            }
            None => {
                println!("\nNo solution found. (search took {:.2?})", duration);
            }
        },
        Err(mpsc::RecvTimeoutError::Timeout) => {
            eprintln!(
                "\nSolver timed out after 60 seconds. The puzzle may be too complex or there might be an infinite loop."
            );
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            eprintln!(
                "\nSolver thread panicked. This indicates a critical bug in the solver logic."
            );
        }
    }

    Ok(())
}
