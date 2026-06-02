use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "sheat",
    about = "Structure Heatmap: visualize file size distribution in a codebase"
)]
pub struct Cli {
    /// Directory or file to analyze
    pub target: Option<PathBuf>,

    /// Output format: markdown, json, or html
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<String>,

    /// Write output to this file (default: stdout)
    #[arg(long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Exclude paths matching this pattern (repeatable)
    #[arg(long = "ignore", value_name = "PATTERN")]
    pub ignore: Vec<String>,

    /// Include hidden files and directories
    #[arg(long)]
    pub include_hidden: bool,

    /// Line counting mode: physical or sloc
    #[arg(long, value_name = "MODE")]
    pub line_mode: Option<String>,

    /// Minimum ratio to report a hotspot
    #[arg(long, value_name = "NUMBER")]
    pub min_ratio: Option<f64>,

    /// Maximum directory depth to aggregate
    #[arg(long, value_name = "NUMBER")]
    pub max_depth: Option<usize>,

    /// Path to configuration file
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
}
