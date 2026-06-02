use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    Markdown,
    Json,
    Html,
}

impl std::str::FromStr for Format {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "markdown" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            "html" => Ok(Self::Html),
            other => Err(format!("unknown format: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineMode {
    Physical,
    Sloc,
}

impl std::str::FromStr for LineMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "physical" => Ok(Self::Physical),
            "sloc" => Ok(Self::Sloc),
            other => Err(format!("unknown line-mode: {other}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub target: PathBuf,
    pub format: Format,
    pub output: Option<PathBuf>,
    pub ignore: Vec<String>,
    pub include_hidden: bool,
    pub line_mode: LineMode,
    pub min_ratio: f64,
    pub max_depth: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target: PathBuf::from("."),
            format: Format::Markdown,
            output: None,
            ignore: Vec::new(),
            include_hidden: false,
            line_mode: LineMode::Physical,
            min_ratio: 3.0,
            max_depth: None,
        }
    }
}

pub fn load_config_file(path: &std::path::Path) -> Result<FileConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read config file {}: {}", path.display(), e))?;
    let base = path.parent().unwrap_or(std::path::Path::new("."));
    let mut fc = parse_config_file(&content)?;
    if let Some(t) = fc.target {
        fc.target = Some(base.join(t));
    }
    if let Some(o) = fc.output {
        fc.output = Some(base.join(o));
    }
    Ok(fc)
}

#[derive(Debug, Default)]
pub struct FileConfig {
    pub target: Option<PathBuf>,
    pub format: Option<Format>,
    pub output: Option<PathBuf>,
    pub ignore: Vec<String>,
    pub include_hidden: Option<bool>,
    pub line_mode: Option<LineMode>,
    pub min_ratio: Option<f64>,
    pub max_depth: Option<usize>,
}

fn parse_config_file(content: &str) -> Result<FileConfig, String> {
    let mut fc = FileConfig::default();
    let known_keys: HashSet<&str> = [
        "target",
        "format",
        "output",
        "ignore",
        "include-hidden",
        "line-mode",
        "min-ratio",
        "max-depth",
    ]
    .iter()
    .copied()
    .collect();

    for (lineno, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, val) = line
            .split_once('=')
            .map(|(k, v)| (k.trim(), v.trim()))
            .ok_or_else(|| format!("line {}: invalid config line: {line}", lineno + 1))?;
        if !known_keys.contains(key) {
            return Err(format!("unknown configuration key: {key}"));
        }
        match key {
            "target" => fc.target = Some(PathBuf::from(val)),
            "format" => {
                fc.format = Some(
                    val.parse()
                        .map_err(|e| format!("line {}: {e}", lineno + 1))?,
                )
            }
            "output" => fc.output = Some(PathBuf::from(val)),
            "ignore" => fc.ignore.push(val.to_string()),
            "include-hidden" => {
                fc.include_hidden = Some(match val {
                    "true" => true,
                    "false" => false,
                    other => return Err(format!("line {}: invalid boolean: {other}", lineno + 1)),
                });
            }
            "line-mode" => {
                fc.line_mode = Some(
                    val.parse()
                        .map_err(|e| format!("line {}: {e}", lineno + 1))?,
                )
            }
            "min-ratio" => {
                let v: f64 = val.parse().map_err(|_| {
                    format!("line {}: invalid number for min-ratio: {val}", lineno + 1)
                })?;
                if v <= 0.0 {
                    return Err(format!("line {}: min-ratio must be positive", lineno + 1));
                }
                fc.min_ratio = Some(v);
            }
            "max-depth" => {
                let v: usize = val.parse().map_err(|_| {
                    format!("line {}: invalid integer for max-depth: {val}", lineno + 1)
                })?;
                fc.max_depth = Some(v);
            }
            _ => unreachable!(),
        }
    }
    Ok(fc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_config() {
        let cfg = parse_config_file("target = .\nformat = json\nmin-ratio = 2.5\nmax-depth = 4\n")
            .unwrap();
        assert_eq!(cfg.target, Some(PathBuf::from(".")));
        assert!(matches!(cfg.format, Some(Format::Json)));
        assert_eq!(cfg.min_ratio, Some(2.5));
        assert_eq!(cfg.max_depth, Some(4));
    }

    #[test]
    fn parse_repeated_ignore() {
        let cfg = parse_config_file("ignore = node_modules\nignore = dist\n").unwrap();
        assert_eq!(cfg.ignore, vec!["node_modules", "dist"]);
    }

    #[test]
    fn parse_unknown_key_fails() {
        let result = parse_config_file("unknown-key = value\n");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown configuration key"));
    }

    #[test]
    fn parse_comments_and_blank_lines() {
        let cfg = parse_config_file("# comment\n\nformat = html\n").unwrap();
        assert!(matches!(cfg.format, Some(Format::Html)));
    }

    #[test]
    fn load_config_file_resolves_target_relative_to_config_dir() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join(".sheatrc");
        std::fs::write(&config_path, "target = subdir\noutput = out.html\n").unwrap();

        let fc = load_config_file(&config_path).unwrap();
        assert_eq!(fc.target, Some(dir.path().join("subdir")));
        assert_eq!(fc.output, Some(dir.path().join("out.html")));
    }

    #[test]
    fn load_config_file_without_paths_leaves_them_none() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join(".sheatrc");
        std::fs::write(&config_path, "min-ratio = 5\n").unwrap();

        let fc = load_config_file(&config_path).unwrap();
        assert_eq!(fc.target, None);
        assert_eq!(fc.output, None);
        assert_eq!(fc.min_ratio, Some(5.0));
    }
}
