use serde::Serialize;

use crate::aggregate::DirAggregate;
use crate::config::{Config, Format, LineMode};
use crate::hotspot::{Hotspot, HotspotKind};

#[derive(Serialize)]
struct JsonOutput<'a> {
    options: JsonOptions<'a>,
    directories: Vec<JsonDir<'a>>,
    hotspots: Vec<JsonHotspot<'a>>,
}

#[derive(Serialize)]
struct JsonOptions<'a> {
    target: &'a str,
    format: &'static str,
    line_mode: &'static str,
    min_ratio: f64,
    max_depth: Option<usize>,
    include_hidden: bool,
    ignore: &'a [String],
}

#[derive(Serialize)]
struct JsonDir<'a> {
    path: &'a str,
    files: usize,
    lines: usize,
}

#[derive(Serialize)]
struct JsonHotspot<'a> {
    path: &'a str,
    kind: &'static str,
    lines: usize,
    baseline: f64,
    ratio: f64,
}

pub fn render(config: &Config, aggregates: &[DirAggregate], hotspots: &[Hotspot]) -> String {
    let format_str = match config.format {
        Format::Markdown => "markdown",
        Format::Json => "json",
        Format::Html => "html",
    };
    let line_mode_str = match config.line_mode {
        LineMode::Physical => "physical",
        LineMode::Sloc => "sloc",
    };

    let output = JsonOutput {
        options: JsonOptions {
            target: &config.target.display().to_string(),
            format: format_str,
            line_mode: line_mode_str,
            min_ratio: config.min_ratio,
            max_depth: config.max_depth,
            include_hidden: config.include_hidden,
            ignore: &config.ignore,
        },
        directories: aggregates
            .iter()
            .map(|a| JsonDir {
                path: if a.path.is_empty() { "." } else { &a.path },
                files: a.files,
                lines: a.lines,
            })
            .collect(),
        hotspots: hotspots
            .iter()
            .map(|h| JsonHotspot {
                path: &h.path,
                kind: match h.kind {
                    HotspotKind::File => "file",
                    HotspotKind::Directory => "directory",
                },
                lines: h.lines,
                baseline: h.baseline,
                ratio: h.ratio,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&output).expect("json serialization failed")
}
