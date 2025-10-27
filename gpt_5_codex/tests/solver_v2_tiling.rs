use pips_solver::{
    loader,
    model::{Game, Placement},
    solver_v2,
};
use std::path::{Path, PathBuf};

fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(relative)
}

fn load_game(path: &str) -> Game {
    let absolute = fixture_path(path);
    loader::load_game_from_path(&absolute).expect("load test game")
}

fn solve_fixture(path: &str) -> Result<Vec<Placement>, String> {
    let game = load_game(path);
    solver_v2::solve(&game)
}

fn solves(path: &str, expected_pieces: usize) {
    let placements = solve_fixture(path).unwrap_or_else(|err| panic!("{} failed: {}", path, err));
    assert_eq!(placements.len(), expected_pieces);
}

#[test]
fn fails_with_unsatisfied_constraint() {
    let result = solve_fixture("poly_games/constraints/domino_impossible.txt");
    assert!(result.is_err(), "expected constraint violation");
}

#[test]
fn solves_2x2() {
    solves("poly_games/2x2.txt", 2);
}

#[test]
fn solves_3x3() {
    solves("poly_games/3x3.txt", 3);
}

#[test]
fn solves_4x4() {
    solves("poly_games/4x4.txt", 8);
}

#[test]
fn solves_2x5() {
    solves("poly_games/2x5.txt", 2);
}
