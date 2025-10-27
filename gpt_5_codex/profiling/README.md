# Profiling Harness

This directory contains a simple wrapper for [`cargo-flamegraph`](https://crates.io/crates/flamegraph). It helps capture flamegraphs for specific puzzle instances that exhibit long runtimes.

## Prerequisites

1. Install the profiler helper:

   ```bash
   cargo install flamegraph
   ```

2. On macOS, `cargo flamegraph` uses `dtrace` and the script will prompt for `sudo`. Allow-list `Terminal` in *System Settings → Privacy & Security → Developer Tools* if prompted.

3. Build the solver in release mode once so the binary is ready:

   ```bash
   cargo build --release --bin pips-solver
   ```

## Usage

From the `gpt_5_codex/` directory:

```bash
./profiling/run_flamegraph.sh game-2025-09-15-hard.txt
```

Key notes:

- The script accepts puzzle paths relative to the repository root or the shared `examples/` directory. If you omit a path, it tries `<repo>/../examples/<name>`.
- Output SVGs default to `profiling/artifacts/<puzzle-name>.svg`. Override with `--output <dir>`.
- Each run produces both the SVG and the raw `perf.data` (or `dtrace` capture) adjacent to the output; inspect the standard output for details.

## Suggested Workflow

1. Profile the known worst cases:

   ```bash
   ./profiling/run_flamegraph.sh \
     game-2025-09-15-hard.txt \
     game-2025-10-14-hard.txt
   ```

2. Open the generated SVGs in a browser to inspect hot paths. Focus on:
   - Constraint reduction routines (`model::constraint::reduce_*`)
   - Game exploration loops (`solver::backtrack`)
   - Loader or I/O (should be minimal after the initial parse)

3. Iterate on solver heuristics, re-run the script, and compare flamegraphs to confirm improvements.
