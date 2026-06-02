# Contributing to Structure Heatmap

Thank you for contributing to Structure Heatmap. This document defines the implementation contract, testing policy, and issue rules for contributors.

## Technical Specification

### Command Contract

The command line entry point is:

```sh
sheat <target>
```

`<target>` may be a file or directory. When omitted, the implementation may use `target` from the configuration file. If neither is available, the command must fail with a non-zero exit code and a clear error message.

Supported options are:

| Option | Type | Default | Contract |
| --- | --- | --- | --- |
| `--format <markdown\|json\|html>` | enum | `markdown` unless configured | Selects exactly one output renderer |
| `--output <path>` | path | stdout | Writes the rendered report to the given path |
| `--ignore <pattern>` | repeatable string | empty | Adds explicit ignore patterns |
| `--include-hidden` | boolean flag | `false` unless configured | Includes hidden files and directories in traversal, except built-in safe exclusions |
| `--line-mode <physical\|sloc>` | enum | `physical` | Selects the line counting algorithm |
| `--min-ratio <number>` | positive number | `3` | Minimum hotspot ratio to include |
| `--max-depth <number>` | non-negative integer | unlimited unless configured | Maximum directory depth to include in aggregation |
| `--config <path>` | path | `.sheatrc` if present | Loads configuration values |

Invalid option values must fail before analysis starts.

### Configuration Contract

The configuration file uses line-oriented `key = value` entries. Blank lines and lines starting with `#` are comments.

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

Supported keys are:

| Key | Type | Repeatable | Maps to |
| --- | --- | --- | --- |
| `target` | path | no | `<target>` |
| `format` | enum | no | `--format` |
| `output` | path | no | `--output` |
| `ignore` | string | yes | `--ignore` |
| `include-hidden` | boolean | no | `--include-hidden` |
| `line-mode` | enum | no | `--line-mode` |
| `min-ratio` | positive number | no | `--min-ratio` |
| `max-depth` | non-negative integer | no | `--max-depth` |

Precedence order is:

1. CLI options
2. Explicit `--config <path>` file
3. Default `.sheatrc` in the current working directory, if present
4. Built-in defaults

Repeated `ignore` values are additive. CLI-provided ignore patterns are added to configuration-provided ignore patterns; they do not replace them.

Unknown configuration keys must fail with a clear error instead of being silently ignored.

### Traversal Contract

Traversal starts from the resolved target.

If the target is a file:

* Analyze that file only.
* Produce a directory summary for its parent directory if the selected output format requires one.
* Hotspot comparison should use only available local context; if no meaningful comparison can be made, no hotspot should be emitted.

If the target is a directory:

* Traverse recursively.
* Include regular files.
* Exclude directories and files matched by ignore rules before counting.
* Exclude hidden directories and files by default. A hidden path is any path with a segment whose file name starts with `.`.
* Include hidden directories and files when `--include-hidden` or `include-hidden = true` is set, except for built-in safe exclusions such as `.git`.
* Do not follow symlinked directories by default.
* Treat unreadable files as errors unless a future option explicitly allows best-effort traversal.

Path handling rules:

* Report paths relative to the target directory when the target is a directory.
* Report the file name when the target is a single file.
* Use `/` as the path separator in reports, including on Windows.
* Sort report rows deterministically by path unless a section explicitly sorts by metric.

### Ignore Contract

Ignored paths are removed from traversal, aggregation, and hotspot calculations.

Ignore sources are:

1. Built-in exclusions required for safe operation, such as `.git`
2. Hidden path exclusion, unless `include-hidden = true` or `--include-hidden` is set
3. `.gitignore` rules discovered at the analyzed root
4. `ignore` entries from configuration
5. `--ignore` entries from the CLI

Explicit ignore patterns use path-style matching against normalized relative paths. Directory patterns must exclude the directory and all descendants.

### Counting Model

The analyzer produces one file metric record per included file:

| Field | Definition |
| --- | --- |
| `path` | Normalized relative path |
| `bytes` | File size in bytes, if available |
| `lines` | Line count according to the selected line mode |
| `line_mode` | `physical` or `sloc` |

`physical` mode:

* Counts newline-delimited physical lines.
* Counts blank lines and comment-only lines.
* A non-empty file without a trailing newline counts as one line.
* An empty file counts as zero lines.

`sloc` mode:

* Excludes blank lines.
* Excludes comment-only lines when the language is recognized. This includes both line comments (e.g. `//`, `#`, `--`) and block comments (e.g. `/* */`, `<!-- -->`, `{- -}`, `(* *)`). A line that contains code in addition to a comment is not excluded.
* Must not exclude inline code only because it contains a trailing comment.
* For unknown languages, the implementation must document whether it falls back to non-blank lines or physical lines.

### Directory Aggregation Model

For each included directory, the analyzer produces an aggregate:

| Field | Definition |
| --- | --- |
| `path` | Normalized relative directory path |
| `files` | Number of included files under the directory, recursively |
| `lines` | Sum of included file line counts under the directory |
| `children` | Immediate included child files and directories, if needed by the renderer |

`max-depth` limits which directory rows are reported. It must not change recursive file counting inside a reported ancestor unless the option is explicitly documented to do so.

Depth is measured from the target root:

* Root directory depth is `0`.
* Immediate children are depth `1`.
* A `max-depth` of `0` reports only the root aggregate.

### Hotspot Model

Hotspots are derived after traversal, ignoring, and line counting.

For file hotspots:

* Compare each file's `lines` value against the median `lines` value of included sibling files in the same directory.
* Exclude zero-line siblings from the median if including them would make the ratio undefined or misleading.
* Emit a hotspot when `file.lines / sibling_median >= min-ratio`.
* Do not emit a hotspot when the sibling median is zero or unavailable.

For directory hotspots:

* Compare each directory aggregate against included sibling directory aggregates.
* Use the same median and ratio rules as file hotspots.
* Directory hotspot support may be omitted only if the output clearly documents that hotspots are file-only.

Hotspot records must include at least:

| Field | Definition |
| --- | --- |
| `path` | File or directory path |
| `lines` | Measured line count |
| `baseline` | Median used for comparison |
| `ratio` | `lines / baseline` |
| `kind` | `file` or `directory` |

Ratios should be rounded for display, but JSON should preserve enough precision for automation.

### Output Contract

All output formats must be deterministic for the same input tree and options.

Markdown output:

* Must include a directory summary table.
* Must include a hotspots table when hotspots exist.
* Must render an explicit empty state when no hotspots exist.

JSON output:

* Must include the resolved options used for analysis.
* Must include directory aggregates.
* Must include hotspot records.
* Must be valid UTF-8 JSON.
* Must avoid breaking field names without a documented compatibility note.

HTML output:

* Must be self-contained.
* Must not require external network access.
* Must include the same core data as Markdown and JSON.
* Must escape paths and generated text to avoid HTML injection.

When `--output` is omitted, output is written to stdout. Diagnostic messages must go to stderr.

### Error Contract

The command must exit with a non-zero status for:

* Missing target
* Unknown format or line mode
* Invalid numeric option values
* Missing configuration file when `--config` is explicitly provided
* Unknown configuration keys
* Unreadable target
* Output write failure

Error messages should name the failing option, path, or configuration key.

### Non-Goals

The implementation must not add dependency analysis, circular dependency detection, architecture rule enforcement, linting, complexity scoring, or automatic refactoring suggestions unless a separate accepted design changes the project scope.

## Test Policy

Contributions should include tests that match the risk and scope of the change.

### Required Test Coverage

Add or update tests when changing:

* CLI argument parsing
* Configuration loading or precedence
* Ignore pattern handling
* `.gitignore` handling
* Line counting behavior
* Directory aggregation
* Hotspot calculation
* Markdown, JSON, or HTML output
* Error handling for invalid input

### Recommended Test Types

Use focused unit tests for pure logic such as line counting, ratio calculation, path filtering, and configuration merging.

Use integration tests for CLI behavior, including:

* Running against small fixture directories
* Combining `.sheatrc` and CLI overrides
* Verifying ignored paths are excluded
* Verifying output files are created
* Checking invalid options return clear errors

Use golden tests for report formats when practical. Golden files are especially useful for Markdown, JSON, and HTML output, but keep them small and easy to review.

### Test Commands

When the Rust project files are present, run:

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

If a contribution changes generated reports or examples, also run the relevant `sheat` command manually against a small fixture or sample project and verify the output.

### Fixture Guidelines

Test fixtures should be minimal. Prefer tiny directory trees with obvious expected results over large sample projects.

Fixtures should include edge cases such as:

* Empty files
* Empty directories
* Files with only comments
* Files with only blank lines
* Nested directories
* Ignored directories
* Sibling files with very different sizes

Do not commit generated reports unless they are intentional test fixtures or documentation examples.

## Issue Rules

Use issues for bug reports, feature requests, technical discussions, and documentation improvements.

### Bug Reports

Bug reports should include:

* The command that was run
* The expected behavior
* The actual behavior
* The operating system
* The Structure Heatmap version or commit
* A minimal directory fixture or reproduction steps when possible
* Relevant `.sheatrc`, `.gitignore`, or CLI options

If the bug affects report output, include the smallest output excerpt needed to show the problem.

### Feature Requests

Feature requests should explain:

* The use case
* Why the feature belongs in structural size analysis
* The expected CLI, configuration, or output behavior
* Any compatibility concerns for existing reports or JSON consumers

Requests for dependency analysis, linting, complexity scoring, or automatic refactoring are usually out of scope unless they are reframed as structural size distribution features.

### Documentation Issues

Documentation issues should identify:

* The unclear or incorrect section
* The suggested correction
* Whether the change should apply to English docs, Japanese docs, or both

### Issue Hygiene

Before opening an issue:

* Search existing issues to avoid duplicates.
* Use a clear title that describes the behavior or proposal.
* Keep one topic per issue.
* Add reproduction details instead of only screenshots when possible.

Maintainers may close issues that are duplicates, out of scope, not reproducible, or inactive after requested information is not provided.

## Pull Request Expectations

Pull requests should:

* Keep changes focused on one topic.
* Update documentation when user-facing behavior changes.
* Add or update tests for behavior changes.
* Avoid unrelated refactors.
* Explain compatibility impacts, especially for JSON output.
* Include the commands used to verify the change.

For documentation-only changes, tests are usually not required, but the changed document should be proofread for consistency with existing project terminology.
