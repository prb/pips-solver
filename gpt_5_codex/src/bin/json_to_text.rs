use serde::Deserialize;
use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct PuzzleFile {
    easy: GameDef,
    medium: GameDef,
    hard: GameDef,
}

#[derive(Debug, Deserialize)]
struct GameDef {
    constructors: Option<String>,
    dominoes: Vec<[u8; 2]>,
    regions: Vec<Region>,
    id: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct Region {
    indices: Vec<[u32; 2]>,
    #[serde(default)]
    target: Option<u32>,
    #[serde(rename = "type")]
    kind: String,
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        return Err(format!(
            "Usage: {} <input.json> [output_dir]",
            args.get(0).map(String::as_str).unwrap_or("json_to_text")
        ));
    }

    let input_path = PathBuf::from(&args[1]);
    let output_dir = if args.len() == 3 {
        PathBuf::from(&args[2])
    } else {
        Path::new(".").to_path_buf()
    };

    let data = std::fs::read_to_string(&input_path)
        .map_err(|err| format!("Failed to read {}: {}", input_path.display(), err))?;
    let puzzle: PuzzleFile =
        serde_json::from_str(&data).map_err(|err| format!("Failed to parse JSON: {}", err))?;

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Input filename must have a valid UTF-8 stem".to_string())?;

    write_game(&puzzle.easy, stem, "easy", &output_dir)?;
    write_game(&puzzle.medium, stem, "medium", &output_dir)?;
    write_game(&puzzle.hard, stem, "hard", &output_dir)?;
    Ok(())
}

fn write_game(game: &GameDef, stem: &str, label: &str, output_dir: &Path) -> Result<(), String> {
    let board_points: BTreeSet<(u32, u32)> = game
        .regions
        .iter()
        .flat_map(|r| r.indices.iter().map(|idx| (idx[1], idx[0])))
        .collect();

    if board_points.is_empty() {
        return Err(format!(
            "Game {} ({}) has no board points defined.",
            game.id.unwrap_or_default(),
            label
        ));
    }

    let min_x = board_points.iter().map(|&(x, _)| x).min().unwrap();
    let max_x = board_points.iter().map(|&(x, _)| x).max().unwrap();
    let min_y = board_points.iter().map(|&(_, y)| y).min().unwrap();
    let max_y = board_points.iter().map(|&(_, y)| y).max().unwrap();

    let mut board_lines = Vec::new();
    for y in min_y..=max_y {
        let mut line = String::new();
        for x in min_x..=max_x {
            if board_points.contains(&(x, y)) {
                line.push('#');
            } else {
                line.push(' ');
            }
        }
        board_lines.push(line);
    }

    let pieces_line = if game.dominoes.is_empty() {
        String::new()
    } else {
        game.dominoes
            .iter()
            .map(|pair| format!("{}{}", pair[0], pair[1]))
            .collect::<Vec<_>>()
            .join(",")
    };

    let mut constraint_lines = Vec::new();
    for region in &game.regions {
        if region.kind == "empty" {
            continue;
        }
        let mut points: Vec<(u32, u32)> =
            region.indices.iter().map(|idx| (idx[1], idx[0])).collect();
        points.sort_by_key(|&(x, y)| (y, x));
        let mut point_repr = String::new();
        point_repr.push('{');
        for (idx, (x, y)) in points.iter().enumerate() {
            if idx > 0 {
                point_repr.push(',');
            }
            let _ = write!(point_repr, "({},{})", x, y);
        }
        point_repr.push('}');

        let text = match region.kind.as_str() {
            "equals" => format!("AllSame None {}", point_repr),
            "unequal" => format!("AllDifferent {} {}", "{}", point_repr),
            "sum" => {
                let target = region
                    .target
                    .ok_or_else(|| format!("sum constraint missing target in {} game", label))?;
                format!("Exactly {} {}", target, point_repr)
            }
            "greater" => {
                let target = region.target.ok_or_else(|| {
                    format!("greater constraint missing target in {} game", label)
                })?;
                format!("MoreThan {} {}", target, point_repr)
            }
            "less" => {
                let target = region
                    .target
                    .ok_or_else(|| format!("less constraint missing target in {} game", label))?;
                format!("LessThan {} {}", target, point_repr)
            }
            other => {
                return Err(format!(
                    "Unknown constraint type '{}' in {} game",
                    other, label
                ));
            }
        };
        constraint_lines.push(text);
    }

    let mut output = String::new();
    let id_text = game
        .id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "unknown-id".to_string());
    let constructors = game
        .constructors
        .as_deref()
        .unwrap_or("Unknown constructor");
    writeln!(output, "// id {} ({}) - {}", id_text, label, constructors).unwrap();
    output.push_str("board:\n");
    for line in &board_lines {
        output.push_str(line);
        output.push('\n');
    }
    output.push('\n');
    output.push_str("pieces:\n");
    if !pieces_line.is_empty() {
        output.push_str(&pieces_line);
        output.push('\n');
    }
    output.push('\n');
    output.push_str("constraints:\n");
    for line in &constraint_lines {
        output.push_str(line);
        output.push('\n');
    }
    output.push('\n');

    let filename = format!("{}-{}.txt", stem, label);
    let mut output_path = output_dir.to_path_buf();
    output_path.push(filename);
    std::fs::write(&output_path, output)
        .map_err(|err| format!("Failed to write {}: {}", output_path.display(), err))?;
    Ok(())
}
