use std::path::Path;
use std::process::Command;

#[test]
fn count_solutions_uses_fixture_directory() {
    let binary = env!("CARGO_BIN_EXE_count_solutions");
    let json_dir = Path::new("../json_games");
    assert!(json_dir.is_dir(), "expected fixtures at {:?}", json_dir);

    let output = Command::new(binary)
        .env("NYT_PIPS_JSON_DIR", json_dir)
        .arg("2025-10-17")
        .arg("hard")
        .output()
        .expect("failed to run count-solutions binary");

    assert!(
        output.status.success(),
        "count-solutions exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Counting solutions for 2025-10-17 Hard"),
        "stdout missing banner:\n{}",
        stdout
    );
    assert!(
        stdout.contains("Total solutions:"),
        "stdout missing total count:\n{}",
        stdout
    );
}
