use std::path::Path;
use std::process::Command;

#[test]
fn solve_pips_uses_local_fixture_directory() {
    let binary = env!("CARGO_BIN_EXE_solve_pips");
    let json_dir = Path::new("../json_games");
    assert!(
        json_dir.is_dir(),
        "expected fixture directory at {:?}",
        json_dir
    );

    let output = Command::new(binary)
        .env("NYT_PIPS_JSON_DIR", json_dir)
        .arg("2025-10-17")
        .arg("easy")
        .output()
        .expect("failed to spawn solve_pips");

    assert!(
        output.status.success(),
        "solve_pips exited with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Solving 2025-10-17 Easy"),
        "stdout missing solve banner:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Dominoes:"),
        "stdout missing domino listing:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Found a solution"),
        "stdout missing solve timing line:\n{}",
        stdout
    );
    assert!(
        stdout.contains('┌'),
        "stdout missing ASCII board framing:\n{}",
        stdout
    );
}
