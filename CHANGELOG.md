# Changelog

## 0.1.1 - 2026-06-03

### Changed

- Updated the CLI help message.

## 0.1.0 - 2026-06-02

Initial beta release.

The planned feature set for version 0.1.0 has been implemented, and this
version is being published as a beta release.

### Added

- Codebase structure analysis based on file counts and line counts.
- Hotspot detection for files and directories that are large compared with
  nearby items.
- Markdown, JSON, and HTML output formats.
- Physical line counting and SLOC line counting modes.
- CLI options for output path, ignore patterns, hidden file handling,
  hotspot ratio threshold, and aggregation depth.
- Configuration file support through `.sheatrc`.
- Automatic exclusion of paths listed in `.gitignore`.
