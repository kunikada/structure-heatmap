use crate::aggregate::DirAggregate;
use crate::traverse::{FileRecord, parent_dir};

#[derive(Debug, Clone)]
pub enum HotspotKind {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct Hotspot {
    pub path: String,
    pub lines: usize,
    pub baseline: f64,
    pub ratio: f64,
    pub kind: HotspotKind,
}

pub fn find_hotspots(
    records: &[FileRecord],
    aggregates: &[DirAggregate],
    min_ratio: f64,
) -> Vec<Hotspot> {
    let mut hotspots: Vec<Hotspot> = Vec::new();
    hotspots.extend(file_hotspots(records, min_ratio));
    hotspots.extend(dir_hotspots(aggregates, min_ratio));
    hotspots.sort_by(|a, b| {
        b.ratio
            .partial_cmp(&a.ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hotspots
}

fn file_hotspots(records: &[FileRecord], min_ratio: f64) -> Vec<Hotspot> {
    let mut by_dir: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    for record in records {
        let dir = parent_dir(&record.path).to_string();
        by_dir.entry(dir).or_default().push(record.lines);
    }

    let mut hotspots = Vec::new();
    for record in records {
        let dir = parent_dir(&record.path).to_string();
        if let Some(siblings) = by_dir.get(&dir) {
            if siblings.len() < 2 {
                continue;
            }
            if let Some(median) = median_nonzero(siblings) {
                let ratio = record.lines as f64 / median;
                if ratio >= min_ratio {
                    hotspots.push(Hotspot {
                        path: record.path.clone(),
                        lines: record.lines,
                        baseline: median,
                        ratio,
                        kind: HotspotKind::File,
                    });
                }
            }
        }
    }
    hotspots
}

fn dir_hotspots(aggregates: &[DirAggregate], min_ratio: f64) -> Vec<Hotspot> {
    let mut by_parent: std::collections::HashMap<String, Vec<usize>> =
        std::collections::HashMap::new();
    for agg in aggregates {
        let parent = parent_dir(&agg.path).to_string();
        by_parent.entry(parent).or_default().push(agg.lines);
    }

    let mut hotspots = Vec::new();
    for agg in aggregates {
        if agg.path.is_empty() {
            continue;
        }
        let parent = parent_dir(&agg.path).to_string();
        if let Some(siblings) = by_parent.get(&parent) {
            if siblings.len() < 2 {
                continue;
            }
            if let Some(median) = median_nonzero(siblings) {
                let ratio = agg.lines as f64 / median;
                if ratio >= min_ratio {
                    hotspots.push(Hotspot {
                        path: agg.path.clone(),
                        lines: agg.lines,
                        baseline: median,
                        ratio,
                        kind: HotspotKind::Directory,
                    });
                }
            }
        }
    }
    hotspots
}

fn median_nonzero(values: &[usize]) -> Option<f64> {
    let mut nonzero: Vec<usize> = values.iter().copied().filter(|&v| v > 0).collect();
    if nonzero.is_empty() {
        return None;
    }
    nonzero.sort_unstable();
    let mid = nonzero.len() / 2;
    let median = if nonzero.len().is_multiple_of(2) {
        (nonzero[mid - 1] + nonzero[mid]) as f64 / 2.0
    } else {
        nonzero[mid] as f64
    };
    Some(median)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LineMode;

    fn make_record(path: &str, lines: usize) -> FileRecord {
        FileRecord {
            path: path.to_string(),
            bytes: 0,
            lines,
            line_mode: LineMode::Physical,
        }
    }

    #[test]
    fn detects_large_file() {
        let records = vec![
            make_record("a.rs", 10),
            make_record("b.rs", 10),
            make_record("big.rs", 100),
        ];
        let hotspots = file_hotspots(&records, 3.0);
        assert_eq!(hotspots.len(), 1);
        assert_eq!(hotspots[0].path, "big.rs");
        assert_eq!(hotspots[0].lines, 100);
        assert_eq!(hotspots[0].baseline, 10.0);
    }

    #[test]
    fn no_hotspot_when_below_ratio() {
        let records = vec![make_record("a.rs", 10), make_record("b.rs", 20)];
        let hotspots = file_hotspots(&records, 3.0);
        assert!(hotspots.is_empty());
    }

    #[test]
    fn median_excludes_zeros() {
        assert_eq!(median_nonzero(&[0, 0, 10]), Some(10.0));
        assert_eq!(median_nonzero(&[0, 0, 0]), None);
    }

    #[test]
    fn median_even_count() {
        assert_eq!(median_nonzero(&[2, 4]), Some(3.0));
    }
}
