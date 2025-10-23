use super::load_game_from_reader;
use crate::model::Game;
use chrono::NaiveDate;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use ureq::Error as UreqError;

const DEFAULT_BASE_URL: &str = "https://www.nytimes.com/svc/pips/v1";

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }

    pub fn all() -> [Difficulty; 3] {
        [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
    }
}

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

pub struct NytPuzzle {
    inner: PuzzleFile,
}

impl NytPuzzle {
    pub fn from_json(json: &str) -> Result<Self, String> {
        let inner: PuzzleFile = serde_json::from_str(json)
            .map_err(|err| format!("Failed to parse puzzle JSON: {}", err))?;
        Ok(Self { inner })
    }

    pub fn game(&self, difficulty: Difficulty) -> Result<Game, String> {
        let (def, label) = match difficulty {
            Difficulty::Easy => (&self.inner.easy, "easy"),
            Difficulty::Medium => (&self.inner.medium, "medium"),
            Difficulty::Hard => (&self.inner.hard, "hard"),
        };
        convert_game(def, label)
    }
}

pub fn fetch_puzzle(date: NaiveDate) -> Result<NytPuzzle, String> {
    let json = fetch_puzzle_json(date)?;
    NytPuzzle::from_json(&json)
}

pub fn fetch_puzzle_json(date: NaiveDate) -> Result<String, String> {
    if let Ok(dir) = env::var("NYT_PIPS_JSON_DIR") {
        if !dir.trim().is_empty() {
            return read_from_directory(PathBuf::from(dir), date);
        }
    }

    let base = env::var("NYT_PIPS_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    fetch_from_base(base.trim(), date)
}

fn fetch_from_base(base: &str, date: NaiveDate) -> Result<String, String> {
    if base.starts_with("file://") {
        let path = &base["file://".len()..];
        return read_from_directory(PathBuf::from(path), date);
    }

    let path_candidate = Path::new(base);
    if path_candidate.is_dir() {
        return read_from_directory(path_candidate.to_path_buf(), date);
    }

    fetch_remote(base, date)
}

fn read_from_directory(directory: PathBuf, date: NaiveDate) -> Result<String, String> {
    let mut path = directory;
    path.push(format!("game-{}.json", date.format("%Y-%m-%d")));
    std::fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read {}: {}", path.display(), err))
}

fn fetch_remote(base_url: &str, date: NaiveDate) -> Result<String, String> {
    let normalized = base_url.trim_end_matches('/');
    let url = format!("{}/{}.json", normalized, date.format("%Y-%m-%d"));
    match ureq::get(&url).call() {
        Ok(response) => response
            .into_string()
            .map_err(|err| format!("Failed to read response from {}: {}", url, err)),
        Err(UreqError::Status(code, _)) => {
            Err(format!("NYTimes returned HTTP {} for {}.", code, url))
        }
        Err(UreqError::Transport(err)) => Err(format!("Request to {} failed: {}", url, err)),
    }
}

fn convert_game(game: &GameDef, label: &str) -> Result<Game, String> {
    let board_points: BTreeSet<(u32, u32)> = game
        .regions
        .iter()
        .flat_map(|region| region.indices.iter().map(|idx| (idx[1], idx[0])))
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
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if board_points.contains(&(x, y)) {
                output.push('#');
            } else {
                output.push(' ');
            }
        }
        output.push('\n');
    }
    output.push('\n');

    output.push_str("pieces:\n");
    if !game.dominoes.is_empty() {
        let pieces = game
            .dominoes
            .iter()
            .map(|pair| format!("{}{}", pair[0], pair[1]))
            .collect::<Vec<_>>()
            .join(",");
        writeln!(output, "{}", pieces).unwrap();
    }
    output.push('\n');

    output.push_str("constraints:\n");
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
        output.push_str(&text);
        output.push('\n');
    }
    output.push('\n');

    load_game_from_reader(Cursor::new(output))
}

#[cfg(test)]
mod tests {
    use super::{Difficulty, NytPuzzle, fetch_puzzle_json};
    use chrono::NaiveDate;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    const SAMPLE_JSON: &str = r#"
{
  "easy": {
    "constructors": "Unit Tester",
    "dominoes": [[1, 2], [2, 3]],
    "regions": [
      {"indices": [[0, 0], [1, 0]], "target": 5, "type": "sum"},
      {"indices": [[0, 1], [1, 1]], "type": "equals"}
    ],
    "id": 10
  },
  "medium": {
    "constructors": null,
    "dominoes": [[3, 4]],
    "regions": [
      {"indices": [[0, 0], [0, 1]], "type": "unequal"}
    ],
    "id": 11
  },
  "hard": {
    "constructors": "Unit Tester",
    "dominoes": [[4, 4]],
    "regions": [
      {"indices": [[0, 0]], "target": 4, "type": "greater"},
      {"indices": [[1, 0]], "target": 6, "type": "less"}
    ],
    "id": 12
  }
}
"#;

    #[test]
    fn parses_sample_puzzle() {
        let puzzle = NytPuzzle::from_json(SAMPLE_JSON).expect("puzzle parses");
        let easy = puzzle.game(Difficulty::Easy).expect("easy game");
        assert_eq!(easy.pieces.len(), 2);
        let medium = puzzle.game(Difficulty::Medium).expect("medium game");
        assert_eq!(medium.pieces.len(), 1);
        let hard = puzzle.game(Difficulty::Hard).expect("hard game");
        assert_eq!(hard.pieces.len(), 1);
    }

    #[test]
    fn fetch_prefers_json_directory_env() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time ok")
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("pips_nyt_{}", timestamp));
        fs::create_dir(&temp_dir).expect("create temp dir");

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let file_path = temp_dir.join("game-2025-01-01.json");
        fs::write(&file_path, SAMPLE_JSON).expect("write sample");

        let guard = EnvGuard::set("NYT_PIPS_JSON_DIR", &temp_dir);
        let json = fetch_puzzle_json(date).expect("fetch from dir");
        drop(guard);

        fs::remove_file(&file_path).ok();
        fs::remove_dir(&temp_dir).ok();
        assert!(json.contains("\"easy\""));
    }

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set<T: AsRef<std::ffi::OsStr>>(key: &'static str, value: T) -> Self {
            let previous = std::env::var(key).ok();
            // Safety: these tests run in process isolation, and we restore the
            // previous value (if any) before the guard drops.
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(prev) = &self.previous {
                unsafe {
                    std::env::set_var(self.key, prev);
                }
            } else {
                unsafe {
                    std::env::remove_var(self.key);
                }
            }
        }
    }
}
