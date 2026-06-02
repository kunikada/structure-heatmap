mod aggregate;
mod cli;
mod config;
mod count;
mod hotspot;
mod output;
mod traverse;

use clap::Parser;
use std::io::Write;

use cli::Cli;
use config::{Config, FileConfig, Format, LineMode, load_config_file};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    let file_config = resolve_config_file(&cli)?;

    let config = build_config(cli, file_config)?;

    let records = traverse::walk(&config)?;
    let aggregates = aggregate::aggregate(&records, config.max_depth);
    let hotspots = hotspot::find_hotspots(&records, &aggregates, config.min_ratio);

    let rendered = match config.format {
        Format::Markdown => output::markdown::render(&aggregates, &hotspots),
        Format::Json => output::json::render(&config, &aggregates, &hotspots),
        Format::Html => output::html::render(&aggregates, &hotspots),
    };

    match &config.output {
        Some(path) => {
            std::fs::write(path, rendered.as_bytes())
                .map_err(|e| format!("cannot write to {}: {e}", path.display()))?;
        }
        None => {
            std::io::stdout()
                .write_all(rendered.as_bytes())
                .map_err(|e| format!("write error: {e}"))?;
        }
    }

    Ok(())
}

fn resolve_config_file(cli: &Cli) -> Result<Option<FileConfig>, String> {
    if let Some(path) = &cli.config {
        if !path.exists() {
            return Err(format!("config file not found: {}", path.display()));
        }
        return Ok(Some(load_config_file(path)?));
    }
    let default_path = std::path::Path::new(".sheatrc");
    if default_path.exists() {
        return Ok(Some(load_config_file(default_path)?));
    }
    Ok(None)
}

fn build_config(cli: Cli, file_config: Option<FileConfig>) -> Result<Config, String> {
    let fc = file_config.unwrap_or_default();

    let target = cli
        .target
        .or(fc.target)
        .ok_or("missing target: specify a directory or file to analyze")?;

    let format = if let Some(s) = cli.format {
        s.parse::<Format>()?
    } else {
        fc.format.unwrap_or(Format::Markdown)
    };

    let line_mode = if let Some(s) = cli.line_mode {
        s.parse::<LineMode>()?
    } else {
        fc.line_mode.unwrap_or(LineMode::Physical)
    };

    let min_ratio = if let Some(v) = cli.min_ratio {
        if v <= 0.0 {
            return Err("--min-ratio must be a positive number".to_string());
        }
        v
    } else {
        fc.min_ratio.unwrap_or(3.0)
    };

    let max_depth = cli.max_depth.or(fc.max_depth);

    let include_hidden = if cli.include_hidden {
        true
    } else {
        fc.include_hidden.unwrap_or(false)
    };

    let mut ignore = fc.ignore;
    ignore.extend(cli.ignore);

    Ok(Config {
        target,
        format,
        output: cli.output.or(fc.output),
        ignore,
        include_hidden,
        line_mode,
        min_ratio,
        max_depth,
    })
}
