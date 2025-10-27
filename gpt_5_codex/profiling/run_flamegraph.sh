#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to run this script." >&2
  exit 1
fi

if ! command -v cargo-flamegraph >/dev/null 2>&1; then
  echo "cargo-flamegraph is not installed. Install it with 'cargo install flamegraph'." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PROFILING_DIR="${SCRIPT_DIR}/artifacts"
EXAMPLES_DIR="${REPO_ROOT}/../examples"
TARGET_PUZZLES=()

usage() {
  cat <<EOF
Usage: $(basename "$0") [--output DIR] puzzle1.txt [puzzle2.txt ...]

Runs cargo flamegraph against the release build of the solver for the provided puzzle files.

Options:
  --output DIR   Directory to store flamegraph outputs (default: profiling/artifacts)

Each flamegraph will be written as DIR/<puzzle-name>.svg.

Examples:
  $(basename "$0") --output ./profiling/artifacts game-2025-09-15-hard.txt
  $(basename "$0") game-2025-10-14-hard.txt game-2025-09-05-hard.txt
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output)
      shift
      [[ $# -gt 0 ]] || { echo "Missing directory after --output"; exit 1; }
      PROFILING_DIR="$1"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    -*)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
    *)
      TARGET_PUZZLES+=("$1")
      shift
      ;;
  esac
done

if [[ ${#TARGET_PUZZLES[@]} -eq 0 ]]; then
  echo "No puzzles supplied." >&2
  usage
  exit 1
fi

mkdir -p "${PROFILING_DIR}"
PROFILING_DIR="$(cd "${PROFILING_DIR}" && pwd)"

pushd "${REPO_ROOT}" >/dev/null

for puzzle in "${TARGET_PUZZLES[@]}"; do
  PUZZLE_PATH="${puzzle}"
  if [[ ! -f "${PUZZLE_PATH}" ]]; then
    if [[ -f "${EXAMPLES_DIR}/${puzzle}" ]]; then
      PUZZLE_PATH="${EXAMPLES_DIR}/${puzzle}"
    else
      echo "Skipping ${puzzle}: file not found locally or in ${EXAMPLES_DIR}" >&2
      continue
    fi
  fi

  OUTPUT="${PROFILING_DIR}/$(basename "${PUZZLE_PATH}" .txt).svg"

  echo "Profiling ${PUZZLE_PATH} -> ${OUTPUT}"

  cargo flamegraph \
    --root \
    --bin pips-solver \
    --release \
    --output "${OUTPUT}" \
    -- "${PUZZLE_PATH}"
done

popd >/dev/null
