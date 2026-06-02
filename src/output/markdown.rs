use crate::aggregate::DirAggregate;
use crate::hotspot::{Hotspot, HotspotKind};

pub fn render(aggregates: &[DirAggregate], hotspots: &[Hotspot]) -> String {
    let mut out = String::new();

    out.push_str("## Directory Summary\n\n");
    out.push_str("| Path | Files | Lines |\n");
    out.push_str("| --- | ---: | ---: |\n");
    for agg in aggregates {
        let display = if agg.path.is_empty() { "." } else { &agg.path };
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            display,
            format_num(agg.files),
            format_num(agg.lines)
        ));
    }

    out.push('\n');
    out.push_str("## Hotspots\n\n");

    if hotspots.is_empty() {
        out.push_str("No hotspots found.\n");
    } else {
        out.push_str("| Path | Kind | Lines | Median | Ratio |\n");
        out.push_str("| --- | --- | ---: | ---: | ---: |\n");
        for h in hotspots {
            let kind = match h.kind {
                HotspotKind::File => "file",
                HotspotKind::Directory => "dir",
            };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {:.1}x |\n",
                h.path,
                kind,
                format_num(h.lines),
                format_num(h.baseline as usize),
                h.ratio
            ));
        }
    }

    out
}

fn format_num(n: usize) -> String {
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
    fn format_num_basic() {
        assert_eq!(format_num(0), "0");
        assert_eq!(format_num(999), "999");
        assert_eq!(format_num(1000), "1,000");
        assert_eq!(format_num(1234567), "1,234,567");
    }

    #[test]
    fn render_no_hotspots_message() {
        let aggs = vec![DirAggregate {
            path: "".to_string(),
            files: 1,
            lines: 10,
        }];
        let out = render(&aggs, &[]);
        assert!(out.contains("No hotspots found."));
    }

    #[test]
    fn render_includes_hotspot_row() {
        use crate::hotspot::HotspotKind;
        let aggs = vec![DirAggregate {
            path: "src".to_string(),
            files: 2,
            lines: 100,
        }];
        let hotspots = vec![Hotspot {
            path: "src/big.rs".to_string(),
            lines: 90,
            baseline: 10.0,
            ratio: 9.0,
            kind: HotspotKind::File,
        }];
        let out = render(&aggs, &hotspots);
        assert!(out.contains("src/big.rs"));
        assert!(out.contains("9.0x"));
    }
}
