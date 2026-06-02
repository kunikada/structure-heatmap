use std::path::Path;

use ignore::WalkBuilder;

use crate::config::{Config, LineMode};
use crate::count::count_lines;

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub path: String,
    #[allow(dead_code)]
    pub bytes: u64,
    pub lines: usize,
    #[allow(dead_code)]
    pub line_mode: LineMode,
}

pub fn walk(config: &Config) -> Result<Vec<FileRecord>, String> {
    let target = &config.target;

    if target.is_file() {
        return walk_single_file(target, config);
    }

    if !target.is_dir() {
        return Err(format!("target not found: {}", target.display()));
    }

    let mut builder = WalkBuilder::new(target);
    builder
        .hidden(!config.include_hidden)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .follow_links(false);

    // Always exclude .git
    builder.filter_entry(|entry| {
        let name = entry.file_name().to_string_lossy();
        name != ".git"
    });

    let mut overrides = ignore::overrides::OverrideBuilder::new(target);
    for pattern in &config.ignore {
        overrides
            .add(&format!("!{pattern}"))
            .map_err(|e| format!("invalid ignore pattern {pattern}: {e}"))?;
        overrides
            .add(&format!("!{pattern}/**"))
            .map_err(|e| format!("invalid ignore pattern {pattern}: {e}"))?;
    }
    let overrides = overrides.build().map_err(|e| e.to_string())?;
    builder.overrides(overrides);

    let mut records: Vec<FileRecord> = Vec::new();

    for result in builder.build() {
        let entry = result.map_err(|e| e.to_string())?;
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            let abs_path = entry.path();
            let rel = relative_path(target, abs_path)?;
            let record = read_file(abs_path, rel, &config.line_mode)?;
            records.push(record);
        }
    }

    records.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(records)
}

fn walk_single_file(path: &Path, config: &Config) -> Result<Vec<FileRecord>, String> {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let record = read_file(path, file_name, &config.line_mode)?;
    Ok(vec![record])
}

fn read_file(path: &Path, rel: String, line_mode: &LineMode) -> Result<FileRecord, String> {
    let content =
        std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let bytes = content.len() as u64;
    let ext = path.extension().and_then(|e| e.to_str());
    let lines = count_lines(&content, line_mode, ext);
    Ok(FileRecord {
        path: rel,
        bytes,
        lines,
        line_mode: line_mode.clone(),
    })
}

fn relative_path(base: &Path, path: &Path) -> Result<String, String> {
    let rel = path
        .strip_prefix(base)
        .map_err(|_| format!("path {} is not under {}", path.display(), base.display()))?;
    Ok(rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

pub fn parent_dir(path: &str) -> &str {
    match path.rfind('/') {
        Some(i) => &path[..i],
        None => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn make_config(target: PathBuf) -> Config {
        Config {
            target,
            ..Default::default()
        }
    }

    #[test]
    fn walk_counts_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), b"line1\nline2\n").unwrap();
        fs::write(dir.path().join("b.txt"), b"only\n").unwrap();
        let records = walk(&make_config(dir.path().to_path_buf())).unwrap();
        assert_eq!(records.len(), 2);
        let a = records.iter().find(|r| r.path == "a.txt").unwrap();
        assert_eq!(a.lines, 2);
    }

    #[test]
    fn walk_excludes_hidden_by_default() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("visible.txt"), b"x\n").unwrap();
        fs::write(dir.path().join(".hidden.txt"), b"x\n").unwrap();
        let records = walk(&make_config(dir.path().to_path_buf())).unwrap();
        assert!(records.iter().all(|r| !r.path.starts_with('.')));
    }

    #[test]
    fn walk_includes_hidden_when_configured() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".hidden.txt"), b"x\n").unwrap();
        let mut cfg = make_config(dir.path().to_path_buf());
        cfg.include_hidden = true;
        let records = walk(&cfg).unwrap();
        assert_eq!(records.len(), 1);
    }

    #[test]
    fn walk_respects_ignore_pattern() {
        let dir = tempdir().unwrap();
        let sub = dir.path().join("node_modules");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("pkg.js"), b"x\n").unwrap();
        fs::write(dir.path().join("main.rs"), b"x\n").unwrap();
        let mut cfg = make_config(dir.path().to_path_buf());
        cfg.ignore = vec!["node_modules".to_string()];
        let records = walk(&cfg).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].path, "main.rs");
    }

    #[test]
    fn parent_dir_works() {
        assert_eq!(parent_dir("src/foo/bar.rs"), "src/foo");
        assert_eq!(parent_dir("main.rs"), "");
        assert_eq!(parent_dir("a/b"), "a");
    }
}
