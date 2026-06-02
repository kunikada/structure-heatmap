# Structure Heatmap

Structure Heatmap is a lightweight analysis tool for visualizing the size of files and directories in a codebase and finding structural imbalances.

Instead of evaluating dependencies or complexity, it focuses on one question:

> Where in the codebase are there areas that are unusually large compared with their surroundings?

Large files and directories are not treated as automatically bad. The tool makes it easier to find places that humans should inspect during design reviews or refactoring.

## Features and Use Cases

Structure Heatmap is intended for tasks such as:

* Checking file counts and line counts by directory
* Detecting files and directories that are large compared with their surroundings
* Counting lines with or without blank lines and comment lines
* Understanding oversized areas before a design review
* Giving AI coding assistants context about structural imbalances in a codebase
* Measuring continuously to track changes in architectural weight
* Quickly finding areas in legacy systems where responsibilities may be concentrated

Examples of structural imbalances it detects include:

* Files that are larger than nearby files
* Directories that have grown large compared with sibling directories
* Areas where file counts or line counts are concentrated
* Places where responsibilities tend to accumulate over time

The following analyses are out of scope:

* Dependency analysis
* Circular dependency detection
* Architecture rule enforcement
* Static code analysis
* Linting
* Complexity scoring
* Automatic refactoring suggestions

Existing specialized tools handle those areas. Structure Heatmap focuses on structural size distribution.

## Installation

Download the binary for your OS from GitHub Releases and place it somewhere executable.

If there is no binary for your OS, compile from source.

```sh
git clone <repository-url>
cd structure-heatmap
cargo build --release
```

## Usage

Run the tool with the root directory of the codebase you want to analyze.

```sh
sheat ./src
```

Use `--format` to specify the output format.

```sh
sheat ./src --format markdown
sheat ./src --format json
sheat ./src --format html
```

Use `--line-mode` to specify how lines are counted.

```sh
sheat ./src --line-mode physical
sheat ./src --line-mode sloc
```

`physical` counts lines including blank lines and comment lines. `sloc` counts effective source lines after excluding blank lines and comment-only lines (both line comments and block comments) for recognized languages.

The default is `physical`.

Example of saving a Markdown report to a file:

```sh
sheat ./src --format markdown --output structure-report.md
```

Example of saving a visual HTML report:

```sh
sheat ./src --format html --output structure-report.html
```

Example output:

```markdown
## Directory Summary

| Path | Files | Lines |
| --- | ---: | ---: |
| src/components | 42 | 8,124 |
| src/domain | 18 | 1,327 |

## Hotspots

| Path | Kind | Lines | Median | Ratio |
| --- | --- | ---: | ---: | ---: |
| src/components/UserEditPage.tsx | file | 842 | 176 | 4.8x |
```

## Configuration and Options

Configuration can be provided through CLI options or a configuration file.

Main options:

| Option | Description |
| --- | --- |
| `--format <markdown\|json\|html>` | Specify the output format |
| `--output <path>` | Specify the file to write results to |
| `--ignore <pattern>` | Specify a pattern to exclude from analysis |
| `--include-hidden` | Include hidden files and directories in analysis |
| `--line-mode <physical\|sloc>` | Specify how lines are counted. The default is `physical` |
| `--min-ratio <number>` | Specify the ratio threshold for hotspots |
| `--max-depth <number>` | Specify the directory depth to aggregate |
| `--config <path>` | Specify the path to a configuration file |

Hidden files and directories, such as `.github`, `.vscode`, and `.sheatrc`, are excluded by default because Structure Heatmap focuses on application source structure rather than tool or environment configuration. Use `--include-hidden` when hidden paths should be part of the structural analysis.

Files and directories listed in `.gitignore` are automatically excluded from analysis.

Examples of filtering options:

```sh
sheat . --ignore node_modules --ignore dist
sheat . --include-hidden
```

JSON output is suitable for automated analysis and CI usage.

```sh
sheat . --format json --output structure-report.json
```

HTML output is suitable for readable reports and shareable artifacts.

```sh
sheat . --format html --output structure-report.html
```

### Configuration File

When using a configuration file, place `.sheatrc` in the directory where you run the `sheat` command (the current working directory).

```ini
target = .
format = html
output = structure-report.html
ignore = node_modules
ignore = dist
include-hidden = false
line-mode = physical
min-ratio = 3
max-depth = 5
```

To specify a configuration file explicitly:

```sh
sheat --config .sheatrc
```

When both CLI options and a configuration file are provided, CLI options take precedence.
