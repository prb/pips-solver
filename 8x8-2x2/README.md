# 8x8-2x2 Pentomino Tiling Proof

This program mechanically proves that any 8×8 board with a 2×2 hole can be tiled with the 12 pentominoes (one of each, up to chirality).

## Building

```bash
cargo build --release
```

## Running

### Basic Usage

Display all 10 solutions sequentially (no colors):

```bash
cargo run --release
```

### With Colors

Display solutions with colored pentomino labels:

```bash
cargo run --release -- --color
```

### Pretty Layout

Arrange the output grids in the same pattern as the hole positions, creating a right-justified triangular layout:
- Row 1: 4 grids for holes at (0,0), (1,0), (2,0), (3,0)
- Row 2: 3 grids for holes at (1,1), (2,1), (3,1)
- Row 3: 2 grids for holes at (2,2), (3,2)
- Row 4: 1 grid for hole at (3,3)

Labels are aligned above each grid for easy identification.

```bash
cargo run --release -- --pretty-layout
```

### Combined

Use both colored output and pretty layout:

```bash
cargo run --release -- --color --pretty-layout
```

## Color Mapping

When using `--color`, each pentomino is shown in a distinct color:

- **F** → Red
- **I** → Green
- **L** → Yellow
- **N** → Blue
- **P** → Magenta
- **T** → Cyan
- **U** → Bright Red
- **V** → Bright Green
- **W** → Bright Yellow
- **X** → Bright Blue
- **Y** → Bright Magenta
- **Z** → Bright Cyan

## Implementation

The solver uses a backtracking algorithm that:
1. Finds the first available cell on the board
2. Tries each remaining pentomino at that position
3. For each pentomino, tries all orientations and placements
4. Recursively solves the remaining board
5. Backtracks if no solution is found

All 10 cases are solved in approximately 2 seconds.
