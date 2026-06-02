use std::collections::BTreeMap;

use crate::traverse::{FileRecord, parent_dir};

#[derive(Debug, Clone)]
pub struct DirAggregate {
    pub path: String,
    pub files: usize,
    pub lines: usize,
}

pub fn aggregate(records: &[FileRecord], max_depth: Option<usize>) -> Vec<DirAggregate> {
    let mut map: BTreeMap<String, (usize, usize)> = BTreeMap::new();

    for record in records {
        let mut dir = parent_dir(&record.path).to_string();
        loop {
            let entry = map.entry(dir.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += record.lines;

            if dir.is_empty() {
                break;
            }
            match dir.rfind('/') {
                Some(i) => dir.truncate(i),
                None => dir.clear(),
            }
        }
    }

    let mut result: Vec<DirAggregate> = map
        .into_iter()
        .filter(|(path, _)| {
            if let Some(max) = max_depth {
                depth(path) <= max
            } else {
                true
            }
        })
        .map(|(path, (files, lines))| DirAggregate { path, files, lines })
        .collect();

    result.sort_by(|a, b| a.path.cmp(&b.path));
    result
}

fn depth(path: &str) -> usize {
    if path.is_empty() {
        0
    } else {
        path.chars().filter(|&c| c == '/').count() + 1
    }
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
    fn aggregate_single_dir() {
        let records = vec![make_record("a.rs", 10), make_record("b.rs", 20)];
        let aggs = aggregate(&records, None);
        let root = aggs.iter().find(|a| a.path.is_empty()).unwrap();
        assert_eq!(root.files, 2);
        assert_eq!(root.lines, 30);
    }

    #[test]
    fn aggregate_nested() {
        let records = vec![
            make_record("src/a.rs", 10),
            make_record("src/b.rs", 20),
            make_record("tests/c.rs", 5),
        ];
        let aggs = aggregate(&records, None);
        let root = aggs.iter().find(|a| a.path.is_empty()).unwrap();
        assert_eq!(root.files, 3);
        assert_eq!(root.lines, 35);
        let src = aggs.iter().find(|a| a.path == "src").unwrap();
        assert_eq!(src.files, 2);
        assert_eq!(src.lines, 30);
    }

    #[test]
    fn aggregate_max_depth() {
        let records = vec![make_record("a/b/c.rs", 5)];
        let aggs = aggregate(&records, Some(1));
        // depth 0 = root "", depth 1 = "a"; "a/b" is depth 2 and should be excluded
        assert!(aggs.iter().all(|a| !a.path.contains("a/b")));
    }

    #[test]
    fn depth_fn() {
        assert_eq!(depth(""), 0);
        assert_eq!(depth("src"), 1);
        assert_eq!(depth("src/foo"), 2);
        assert_eq!(depth("src/foo/bar"), 3);
    }
}
