use crate::aggregate::DirAggregate;
use crate::hotspot::{Hotspot, HotspotKind};

pub fn render(aggregates: &[DirAggregate], hotspots: &[Hotspot]) -> String {
    let dir_rows: String = aggregates
        .iter()
        .map(|a| {
            let display = if a.path.is_empty() { "." } else { &a.path };
            format!(
                "<tr><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>\n",
                escape(display),
                fmt_num(a.files),
                fmt_num(a.lines)
            )
        })
        .collect();

    let hotspot_section = if hotspots.is_empty() {
        "<p>No hotspots found.</p>\n".to_string()
    } else {
        let rows: String = hotspots
            .iter()
            .map(|h| {
                let kind = match h.kind {
                    HotspotKind::File => "file",
                    HotspotKind::Directory => "dir",
                };
                format!(
                    "<tr><td>{}</td><td>{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{:.1}x</td></tr>\n",
                    escape(&h.path),
                    kind,
                    fmt_num(h.lines),
                    fmt_num(h.baseline as usize),
                    h.ratio
                )
            })
            .collect();
        format!(
            "<table>\n<thead><tr><th>Path</th><th>Kind</th><th>Lines</th><th>Median</th><th>Ratio</th></tr></thead>\n<tbody>\n{rows}</tbody>\n</table>\n"
        )
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Structure Heatmap Report</title>
<style>
body {{ font-family: monospace; max-width: 900px; margin: 2rem auto; padding: 0 1rem; }}
h2 {{ border-bottom: 1px solid #ccc; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{ border: 1px solid #ddd; padding: 4px 8px; text-align: left; }}
.num {{ text-align: right; }}
tr:nth-child(even) {{ background: #f8f8f8; }}
</style>
</head>
<body>
<h1>Structure Heatmap Report</h1>
<h2>Directory Summary</h2>
<table>
<thead><tr><th>Path</th><th>Files</th><th>Lines</th></tr></thead>
<tbody>
{dir_rows}</tbody>
</table>
<h2>Hotspots</h2>
{hotspot_section}</body>
</html>
"#
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn fmt_num(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_html_chars() {
        assert_eq!(escape("<script>"), "&lt;script&gt;");
        assert_eq!(escape("a & b"), "a &amp; b");
    }

    #[test]
    fn render_is_valid_html() {
        let aggs = vec![DirAggregate {
            path: "".to_string(),
            files: 1,
            lines: 5,
        }];
        let html = render(&aggs, &[]);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("No hotspots found."));
    }
}
