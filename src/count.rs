use crate::config::LineMode;

pub fn count_lines(content: &[u8], mode: &LineMode, ext: Option<&str>) -> usize {
    match mode {
        LineMode::Physical => count_physical(content),
        LineMode::Sloc => count_sloc(content, ext),
    }
}

fn count_physical(content: &[u8]) -> usize {
    if content.is_empty() {
        return 0;
    }
    let newlines = content.iter().filter(|&&b| b == b'\n').count();
    // A file without a trailing newline still counts as one line
    if content.last() == Some(&b'\n') {
        newlines
    } else {
        newlines + 1
    }
}

fn count_sloc(content: &[u8], ext: Option<&str>) -> usize {
    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return count_physical(content),
    };

    let syntax = ext.and_then(comment_syntax);

    match syntax {
        None => {
            // Unknown language: exclude blank lines only.
            text.lines().filter(|l| !l.trim().is_empty()).count()
        }
        Some(syn) => count_sloc_with_syntax(text, syn),
    }
}

/// Returns true if `s` contains real code after consuming all leading comments.
///
/// Scans left-to-right, repeatedly stripping block comments (`block`) and
/// stopping at the first line-comment prefix (`line`).  Whatever non-whitespace
/// remains is considered code.
fn has_code(mut s: &str, syn: &CommentSyntax) -> bool {
    loop {
        s = s.trim();
        if s.is_empty() {
            return false;
        }
        // A line-comment prefix terminates the scan — nothing after it is code.
        if let Some(lp) = syn.line
            && s.starts_with(lp)
        {
            return false;
        }
        // Strip a leading block comment if one opens here.
        if let Some((open, close)) = syn.block
            && s.starts_with(open)
        {
            match s[open.len()..].find(close) {
                Some(rel) => {
                    // Block opens and closes — advance past the closing delimiter.
                    s = &s[open.len() + rel + close.len()..];
                    continue;
                }
                None => {
                    // Block opens but never closes on this segment — no code.
                    return false;
                }
            }
        }
        // Something other than a comment remains.
        return true;
    }
}

fn count_sloc_with_syntax(text: &str, syn: CommentSyntax) -> usize {
    let mut count = 0;
    let mut in_block = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if in_block {
            // Consume the rest of the block comment.
            if let Some((_, end)) = syn.block {
                if let Some(pos) = trimmed.find(end) {
                    in_block = false;
                    // Real code (not just a line comment) after the closing delimiter?
                    let after = trimmed[pos + end.len()..].trim();
                    if has_code(after, &syn) {
                        count += 1;
                    }
                }
                // Whether the block ended or not, this line is comment content.
                continue;
            }
        }

        if trimmed.is_empty() {
            continue;
        }

        // Scan left-to-right: find whichever token comes first — line-comment
        // prefix or block-comment open.  The winner determines how to classify
        // the rest of the line.  This correctly handles cases such as:
        //   `fn f() {} // /* x */`  — line comment wins, no block started
        //   `/* // not a line comment */` — block open wins, no line comment
        let line_pos = syn.line.and_then(|p| trimmed.find(p));
        let block_open_pos = syn.block.and_then(|(s, _)| trimmed.find(s));

        match (line_pos, block_open_pos) {
            // Line comment comes first (or there is no block open).
            (Some(lp), block_op) if block_op.is_none_or(|bp| lp <= bp) => {
                // Everything before the line comment prefix is code.
                if trimmed[..lp].trim().is_empty() {
                    // Pure line-comment line.
                } else {
                    count += 1;
                }
            }
            // Block comment open comes first (or there is no line comment).
            (_, Some(bp)) => {
                let before = trimmed[..bp].trim();
                let has_code_before = !before.is_empty();
                let rest = &trimmed[bp + syn.block.unwrap().0.len()..];

                if let Some(close_rel) = rest.find(syn.block.unwrap().1) {
                    // Block opens and closes on the same line.
                    let after = rest[close_rel + syn.block.unwrap().1.len()..].trim();
                    if has_code_before || has_code(after, &syn) {
                        count += 1;
                    }
                    // If neither before nor after has code, it is a pure comment line.
                } else {
                    // Block opens but does not close on this line.
                    in_block = true;
                    if has_code_before {
                        count += 1;
                    }
                }
            }
            // Line comment with no block open anywhere on the line.
            (Some(lp), None) => {
                if !trimmed[..lp].trim().is_empty() {
                    count += 1;
                }
            }
            // No comment tokens at all — plain code line.
            (None, None) => {
                count += 1;
            }
        }
    }

    count
}

struct CommentSyntax {
    /// Single-line comment prefix, e.g. `//` or `#`.
    line: Option<&'static str>,
    /// Block comment delimiters (open, close), e.g. `("/*", "*/")`.
    block: Option<(&'static str, &'static str)>,
}

/// Returns comment syntax for a recognized extension, or `None` for unknown ones.
fn comment_syntax(ext: &str) -> Option<CommentSyntax> {
    Some(match ext {
        "rs" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "java" | "js" | "jsx" | "ts" | "tsx"
        | "cs" | "go" | "swift" | "kt" | "kts" | "scala" | "dart" | "groovy" | "gradle" => {
            CommentSyntax {
                line: Some("//"),
                block: Some(("/*", "*/")),
            }
        }
        "css" | "scss" | "sass" | "less" => CommentSyntax {
            line: None,
            block: Some(("/*", "*/")),
        },
        "py" | "rb" | "sh" | "bash" | "zsh" | "fish" | "pl" | "r" | "R" | "yml" | "yaml"
        | "toml" | "conf" | "ini" | "tf" | "hcl" => CommentSyntax {
            line: Some("#"),
            block: None,
        },
        "ex" | "exs" => CommentSyntax {
            line: Some("#"),
            block: None,
        },
        "lua" => CommentSyntax {
            line: Some("--"),
            block: Some(("--[[", "]]")),
        },
        "sql" => CommentSyntax {
            line: Some("--"),
            block: Some(("/*", "*/")),
        },
        "hs" | "lhs" => CommentSyntax {
            line: Some("--"),
            block: Some(("{-", "-}")),
        },
        "erl" | "hrl" => CommentSyntax {
            line: Some("%"),
            block: None,
        },
        "ml" | "mli" => CommentSyntax {
            line: None,
            block: Some(("(*", "*)")),
        },
        "vim" => CommentSyntax {
            line: Some("\""),
            block: None,
        },
        "html" | "htm" | "xml" | "svg" | "md" | "markdown" => CommentSyntax {
            line: None,
            block: Some(("<!--", "-->")),
        },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_zero_lines() {
        assert_eq!(count_physical(b""), 0);
        assert_eq!(count_sloc(b"", None), 0);
    }

    #[test]
    fn single_line_no_newline() {
        assert_eq!(count_physical(b"hello"), 1);
    }

    #[test]
    fn single_line_with_newline() {
        assert_eq!(count_physical(b"hello\n"), 1);
    }

    #[test]
    fn multiple_lines() {
        assert_eq!(count_physical(b"a\nb\nc\n"), 3);
        assert_eq!(count_physical(b"a\nb\nc"), 3);
    }

    #[test]
    fn blank_lines_excluded_in_sloc() {
        assert_eq!(count_sloc(b"a\n\nb\n", None), 2);
    }

    #[test]
    fn blank_lines_counted_in_physical() {
        assert_eq!(count_physical(b"a\n\nb\n"), 3);
    }

    #[test]
    fn comment_lines_excluded_in_sloc_known_lang() {
        let src = b"fn main() {}\n// comment\n\n  // indented comment\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn inline_comment_not_excluded() {
        let src = b"let x = 1; // inline\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn unknown_ext_falls_back_to_nonblank() {
        let src = b"# comment\ncode\n\n";
        // unknown extension: # is not stripped, both non-blank lines count
        assert_eq!(count_sloc(src, Some("xyz")), 2);
    }

    #[test]
    fn block_comment_single_line_excluded() {
        let src = b"code\n/* a block comment */\nmore code\n";
        assert_eq!(count_sloc(src, Some("rs")), 2);
    }

    #[test]
    fn block_comment_multiline_excluded() {
        let src = b"code\n/*\n * middle\n */\nmore code\n";
        assert_eq!(count_sloc(src, Some("rs")), 2);
    }

    #[test]
    fn block_comment_inline_not_excluded() {
        // Code precedes the block comment — line counts.
        let src = b"int x = 1; /* set x */\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn block_comment_code_after_close_counts() {
        // Code follows the closing delimiter on the same line.
        let src = b"/* comment */ int x;\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn html_block_comment_excluded() {
        let src = b"<div>\n<!-- comment -->\n</div>\n";
        assert_eq!(count_sloc(src, Some("html")), 2);
    }

    #[test]
    fn haskell_block_comment_excluded() {
        let src = b"main = pure ()\n{-\nnote\n-}\n";
        assert_eq!(count_sloc(src, Some("hs")), 1);
    }

    #[test]
    fn ocaml_block_comment_excluded() {
        let src = b"let x = 1\n(* block *)\nlet y = 2\n";
        assert_eq!(count_sloc(src, Some("ml")), 2);
    }

    #[test]
    fn css_no_line_comment_block_excluded() {
        let src = b".foo { color: red; }\n/* reset */\n.bar {}\n";
        assert_eq!(count_sloc(src, Some("css")), 2);
    }

    #[test]
    fn line_comment_containing_block_delimiter_not_mishandled() {
        // `// /* foo */` must be treated as a line comment, not a block comment start.
        // The lines after it must not be swallowed as block comment body.
        let src = b"// /* not a block start */\nlet x = 1;\nlet y = 2;\n";
        assert_eq!(count_sloc(src, Some("rs")), 2);
    }

    #[test]
    fn line_comment_containing_block_open_does_not_corrupt_state() {
        // Ensure in_block stays false after the line-comment line.
        let src = b"code_a\n// /* open\ncode_b\ncode_c\n";
        assert_eq!(count_sloc(src, Some("rs")), 3);
    }

    #[test]
    fn trailing_line_comment_with_block_delimiter_not_mishandled() {
        // `fn a() {} // /* not block` must not start a block comment.
        // Both lines must count as sloc.
        let src = b"fn a() {} // /* not block\nfn b() {}\n";
        assert_eq!(count_sloc(src, Some("rs")), 2);
    }

    #[test]
    fn trailing_line_comment_block_delimiter_does_not_corrupt_state() {
        // Lines after the trailing-comment line must not be swallowed.
        let src = b"fn a() {} // /*\nfn b() {}\nfn c() {}\n";
        assert_eq!(count_sloc(src, Some("rs")), 3);
    }

    #[test]
    fn block_comment_containing_line_comment_prefix_excluded() {
        // `/* // inside block */` must be treated as a block comment, not a line comment.
        // sloc should be 1 (only `fn b() {}`).
        let src = b"/* // inside block */\nfn b() {}\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn multiline_block_containing_line_prefix_does_not_corrupt_state() {
        // The `//` inside the open block must not terminate scanning early,
        // leaving in_block true and swallowing subsequent lines.
        let src = b"/*\n// still in block\n*/\nfn b() {}\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn block_close_followed_by_line_comment_only_not_counted() {
        // `/* block */ // trailing comment only` is a comment-only line.
        let src = b"/* block */ // trailing comment only\nfn b() {}\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn multiline_block_close_followed_by_line_comment_only_not_counted() {
        // `*/  // trailing` at the end of a multiline block is still comment-only.
        let src = b"/*\nmiddle\n*/ // trailing\nfn b() {}\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn block_close_followed_by_real_code_still_counted() {
        // `/* block */ code` must still count.
        let src = b"/* comment */ int x;\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn multiple_block_comments_on_one_line_not_counted() {
        // `/* a */ /* b */` is comment-only — sloc must be 1 (only `fn c()`).
        let src = b"/* a */ /* b */\nfn c() {}\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn multiline_block_close_then_another_block_not_counted() {
        // `*/ /* b */` closing a multiline block then another inline block is still comment-only.
        let src = b"/*\nmiddle\n*/ /* b */\nfn c() {}\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }

    #[test]
    fn multiple_blocks_then_code_counted() {
        // `/* a */ /* b */ code` has real code after the comments.
        let src = b"/* a */ /* b */ int x;\n";
        assert_eq!(count_sloc(src, Some("rs")), 1);
    }
}
