use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "sheat",
    about = "Count lines per file/directory and flag hotspots that are disproportionately large"
)]
pub struct Cli {
    /// Root directory or file to analyze (falls back to target in .sheatrc)
    pub target: Option<PathBuf>,

    /// Output format [default: markdown] [possible values: markdown, json, html]
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<String>,

    /// Write output to this file instead of stdout
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Glob pattern to exclude paths (repeatable)
    #[arg(long = "ignore", value_name = "PATTERN")]
    pub ignore: Vec<String>,

    /// Include dot-files and dot-directories
    #[arg(long)]
    pub include_hidden: bool,

    /// How to count lines [default: physical] [possible values: physical (all lines), sloc (skip blank lines and comments)]
    #[arg(long, value_name = "MODE")]
    pub line_mode: Option<String>,

    /// Flag a file/directory as a hotspot if its line count is at least N times the median of its siblings [default: 3]
    #[arg(long, value_name = "NUMBER")]
    pub min_ratio: Option<f64>,

    /// Aggregate directories deeper than this level into their parent [default: unlimited]
    #[arg(long, value_name = "NUMBER")]
    pub max_depth: Option<usize>,

    /// Load settings from this file instead of the default .sheatrc
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
}
