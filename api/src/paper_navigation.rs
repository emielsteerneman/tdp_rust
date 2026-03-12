use data_structures::content::TocEntry;
use data_structures::intermediate::BreadcrumbEntry;

/// Returns which content_seq's belong to a section and everything nested inside it.
///
/// Given ToC `[(0,d1,"Vision"), (1,d2,"Camera"), (2,d1,"Motion")]`,
/// `compute_section_range(toc, 0)` → `Some((0, 2))` (Vision owns Camera but not Motion).
pub fn compute_section_range(toc: &[TocEntry], content_seq: u32) -> Option<(u32, u32)> {
    let target_idx = toc.iter().position(|e| e.content_seq == content_seq)?;
    let start_depth = toc[target_idx].depth;

    // Walk forward to find the first sibling or ancestor (depth <= start)
    let end_seq = toc[target_idx + 1..]
        .iter()
        .find(|e| e.depth <= start_depth)
        .map(|e| e.content_seq)
        .unwrap_or(u32::MAX); // no boundary found → section runs to end of paper

    Some((content_seq, end_seq))
}

/// Returns the parent chain from the root down to (but not including) the given section.
///
/// Given ToC `[(0,d1,"Vision"), (1,d2,"Camera"), (2,d3,"Mirror")]`,
/// `compute_breadcrumbs(toc, 2)` → `[Vision, Camera]` (ancestors of Mirror, root-first).
pub fn compute_breadcrumbs(toc: &[TocEntry], content_seq: u32) -> Vec<BreadcrumbEntry> {
    // Find the index and depth of the target entry
    let Some(target_idx) = toc.iter().position(|e| e.content_seq == content_seq) else {
        return Vec::new();
    };
    let target_depth = toc[target_idx].depth;

    // Walk backwards, collecting the nearest ancestor at each strictly lower depth
    let mut crumbs = Vec::new();
    let mut needed_depth = target_depth;

    for entry in toc[..target_idx].iter().rev() {
        if entry.depth < needed_depth {
            crumbs.push(BreadcrumbEntry {
                content_seq: entry.content_seq,
                title: entry.title.clone(),
            });
            needed_depth = entry.depth;
            if needed_depth == 0 {
                break;
            }
        }
    }

    crumbs.reverse();
    crumbs
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_structures::content::ContentType;

    fn make_toc(entries: &[(u32, u8, &str)]) -> Vec<TocEntry> {
        entries
            .iter()
            .map(|(seq, depth, title)| TocEntry {
                content_seq: *seq,
                content_type: ContentType::Text,
                depth: *depth,
                title: title.to_string(),
            })
            .collect()
    }

    #[test]
    fn test_breadcrumbs_root_section() {
        let toc = make_toc(&[(0, 1, "Introduction"), (1, 1, "Methods")]);
        // Root-level section → no breadcrumbs
        let crumbs = compute_breadcrumbs(&toc, 0);
        assert!(crumbs.is_empty());
    }

    #[test]
    fn test_breadcrumbs_one_level_deep() {
        let toc = make_toc(&[
            (0, 1, "Introduction"),
            (1, 2, "Background"),
            (2, 2, "Motivation"),
        ]);
        let crumbs = compute_breadcrumbs(&toc, 1);
        assert_eq!(crumbs.len(), 1);
        assert_eq!(crumbs[0].content_seq, 0);
        assert_eq!(crumbs[0].title, "Introduction");
    }

    #[test]
    fn test_breadcrumbs_two_levels_deep() {
        let toc = make_toc(&[
            (0, 1, "Vision"),
            (1, 2, "Camera"),
            (2, 3, "Mirror"),
            (3, 2, "Detection"),
        ]);
        let crumbs = compute_breadcrumbs(&toc, 2);
        assert_eq!(crumbs.len(), 2);
        assert_eq!(crumbs[0].title, "Vision");
        assert_eq!(crumbs[1].title, "Camera");
    }

    #[test]
    fn test_breadcrumbs_not_found() {
        let toc = make_toc(&[(0, 1, "Introduction")]);
        let crumbs = compute_breadcrumbs(&toc, 99);
        assert!(crumbs.is_empty());
    }

    #[test]
    fn test_breadcrumbs_skip_siblings() {
        // Ensure we pick the nearest ancestor, not a sibling
        let toc = make_toc(&[
            (0, 1, "Hardware"),
            (1, 2, "Motors"),
            (2, 2, "Sensors"),
            (3, 3, "Camera"),
        ]);
        let crumbs = compute_breadcrumbs(&toc, 3);
        assert_eq!(crumbs.len(), 2);
        assert_eq!(crumbs[0].title, "Hardware");
        assert_eq!(crumbs[1].title, "Sensors"); // nearest depth=2, not Motors
    }

    #[test]
    fn test_breadcrumbs_empty_toc() {
        let crumbs = compute_breadcrumbs(&[], 0);
        assert!(crumbs.is_empty());
    }

    // ── compute_section_range tests ──────────────────────────────────────

    #[test]
    fn test_section_range_with_children() {
        let toc = make_toc(&[
            (0, 1, "Vision"),
            (1, 2, "Camera"),
            (2, 3, "Mirror"),
            (3, 3, "Specs"),
            (4, 2, "Detection"),
        ]);
        // "Camera" (seq=1, depth=2) owns seq 1..4 (stops at "Detection" depth=2)
        assert_eq!(compute_section_range(&toc, 1), Some((1, 4)));
    }

    #[test]
    fn test_section_range_leaf_node() {
        let toc = make_toc(&[
            (0, 1, "Vision"),
            (1, 2, "Camera"),
            (2, 2, "Detection"),
        ]);
        // "Camera" has no children, next sibling is "Detection"
        assert_eq!(compute_section_range(&toc, 1), Some((1, 2)));
    }

    #[test]
    fn test_section_range_last_section() {
        let toc = make_toc(&[
            (0, 1, "Vision"),
            (1, 2, "Camera"),
            (2, 3, "Mirror"),
        ]);
        // "Camera" runs to end of paper
        assert_eq!(compute_section_range(&toc, 1), Some((1, u32::MAX)));
    }

    #[test]
    fn test_section_range_root() {
        let toc = make_toc(&[
            (0, 1, "Vision"),
            (1, 2, "Camera"),
            (2, 1, "Motion"),
        ]);
        // "Vision" (depth=1) owns seq 0..2
        assert_eq!(compute_section_range(&toc, 0), Some((0, 2)));
    }

    #[test]
    fn test_section_range_not_found() {
        let toc = make_toc(&[(0, 1, "Vision")]);
        assert_eq!(compute_section_range(&toc, 99), None);
    }
}
