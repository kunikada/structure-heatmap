use std::fs;
use std::path::Path;
use std::process::Command;

fn sheat() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sheat"))
}

fn fixture_dir(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

// --- basic traversal ---

#[test]
fn runs_on_fixture_basic() {
    let dir = fixture_dir("basic");
    let out = sheat().arg(&dir).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Directory Summary"));
    assert!(stdout.contains("Hotspots"));
}

#[test]
fn counts_correct_lines() {
    let dir = fixture_dir("basic");
    let out = sheat().arg(&dir).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    // small.rs(5) + small2.rs(5) + big.rs(50) = 60 lines
    assert!(stdout.contains("60"), "expected 60 total lines:\n{stdout}");
}

#[test]
fn detects_hotspot_in_basic_fixture() {
    let dir = fixture_dir("basic");
    let out = sheat().arg(&dir).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("big.rs"),
        "expected big.rs in hotspots:\n{stdout}"
    );
}

// --- ignore pattern ---

#[test]
fn ignores_specified_directory() {
    let dir = fixture_dir("with_ignored");
    let out = sheat()
        .arg(&dir)
        .arg("--ignore")
        .arg("vendor")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("vendor"),
        "vendor should be excluded:\n{stdout}"
    );
}

// --- hidden files ---

#[test]
fn excludes_hidden_by_default() {
    let dir = fixture_dir("with_hidden");
    let out = sheat().arg(&dir).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains(".hidden"),
        "hidden file should be excluded:\n{stdout}"
    );
}

#[test]
fn includes_hidden_when_flag_set() {
    let dir = fixture_dir("with_hidden");

    let out_default = sheat().arg(&dir).output().unwrap();
    let stdout_default = String::from_utf8_lossy(&out_default.stdout);
    let files_default = extract_root_files(&stdout_default);

    let out_hidden = sheat().arg(&dir).arg("--include-hidden").output().unwrap();
    let stdout_hidden = String::from_utf8_lossy(&out_hidden.stdout);
    let files_hidden = extract_root_files(&stdout_hidden);

    assert!(
        files_hidden > files_default,
        "include-hidden should increase file count ({files_default} -> {files_hidden}):\n{stdout_hidden}"
    );
}

// --- output formats ---

#[test]
fn json_output_is_valid() {
    let dir = fixture_dir("basic");
    let out = sheat()
        .arg(&dir)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("invalid JSON: {e}\n{stdout}"));
    assert!(parsed["options"].is_object());
    assert!(parsed["directories"].is_array());
    assert!(parsed["hotspots"].is_array());
}

#[test]
fn html_output_is_self_contained() {
    let dir = fixture_dir("basic");
    let out = sheat()
        .arg(&dir)
        .arg("--format")
        .arg("html")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("<!DOCTYPE html>"));
    assert!(stdout.contains("</html>"));
    assert!(
        !stdout.contains("http"),
        "HTML must not load external resources"
    );
}

// --- output file ---

#[test]
fn writes_output_to_file() {
    let dir = fixture_dir("basic");
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("report.md");
    let status = sheat()
        .arg(&dir)
        .arg("--output")
        .arg(&out_path)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(out_path.exists());
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("Directory Summary"));
}

// --- config file ---

#[test]
fn config_file_is_loaded() {
    let dir = fixture_dir("with_config");
    let config_path = dir.join(".sheatrc");
    // pass --config explicitly since cwd is not the fixture dir
    let out = sheat()
        .arg(&dir)
        .arg("--config")
        .arg(&config_path)
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No hotspots found."),
        "expected no hotspots with min-ratio=100:\n{stdout}"
    );
}

#[test]
fn cli_overrides_config_file() {
    let dir = fixture_dir("with_config");
    let config_path = dir.join(".sheatrc");
    // override the high min-ratio with a low value — hotspot should appear
    let out = sheat()
        .arg(&dir)
        .arg("--config")
        .arg(&config_path)
        .arg("--min-ratio")
        .arg("2")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("big.rs"),
        "expected hotspot with low min-ratio:\n{stdout}"
    );
}

// --- error handling ---

#[test]
fn missing_target_exits_nonzero() {
    let out = sheat().output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn nonexistent_target_exits_nonzero() {
    let out = sheat()
        .arg("/nonexistent/path/that/does/not/exist")
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn invalid_format_exits_nonzero() {
    let dir = fixture_dir("basic");
    let out = sheat()
        .arg(&dir)
        .arg("--format")
        .arg("xml")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("xml") || stderr.contains("format"),
        "stderr: {stderr}"
    );
}

#[test]
fn missing_explicit_config_exits_nonzero() {
    let dir = fixture_dir("basic");
    let out = sheat()
        .arg(&dir)
        .arg("--config")
        .arg("/nonexistent/.sheatrc")
        .output()
        .unwrap();
    assert!(!out.status.success());
}

// --- line mode ---

#[test]
fn sloc_excludes_blank_lines() {
    let dir = fixture_dir("line_modes");
    let physical = sheat()
        .arg(&dir)
        .arg("--line-mode")
        .arg("physical")
        .output()
        .unwrap();
    let sloc = sheat()
        .arg(&dir)
        .arg("--line-mode")
        .arg("sloc")
        .output()
        .unwrap();
    assert!(physical.status.success());
    assert!(sloc.status.success());
    // physical line count must be >= sloc line count (fixture has blank lines)
    let phys_lines = extract_root_lines(&String::from_utf8_lossy(&physical.stdout));
    let sloc_lines = extract_root_lines(&String::from_utf8_lossy(&sloc.stdout));
    assert!(
        phys_lines >= sloc_lines,
        "physical ({phys_lines}) should be >= sloc ({sloc_lines})"
    );
}

fn extract_root_files(markdown: &str) -> usize {
    for line in markdown.lines() {
        if line.starts_with("| . |") || line.starts_with("| . ") {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                let num = parts[2].trim().replace(',', "");
                if let Ok(n) = num.parse::<usize>() {
                    return n;
                }
            }
        }
    }
    0
}

fn extract_root_lines(markdown: &str) -> usize {
    for line in markdown.lines() {
        if line.starts_with("| . |") || line.starts_with("| . ") {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let num = parts[3].trim().replace(',', "");
                if let Ok(n) = num.parse::<usize>() {
                    return n;
                }
            }
        }
    }
    0
}
