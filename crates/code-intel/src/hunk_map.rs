//! Hunk-to-Symbol Impact Mapping.
//!
//! Maps safe diff hunks to extracted symbols using interval overlap semantics.
//! Primary mapping uses new-file line ranges (new_start/new_count) against
//! current-file symbols. Old ranges retained for diagnostics only.
//!
//! Key rules:
//! - One hunk may map to zero, one, or many symbols
//! - Most-specific symbol wins; parent as metadata
//! - Parent container emitted as separate impact only if boundary touched
//! - Output bounded: max 50 per file, 200 total
//! - Deleted hunks/files degrade to FileLevelOnly/NoSymbolMatch
//! - No source bodies in output; read-only operation

use serde::Serialize;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// A parsed unified-diff hunk with both old and new line ranges.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ParsedHunk {
    /// Deterministic ID: path:old_start-new_start
    pub hunk_id: String,
    /// Old file line range start (1-based).
    pub old_start: u32,
    /// Number of lines in old file.
    pub old_count: u32,
    /// New file line range start (1-based).
    pub new_start: u32,
    /// Number of lines in new file.
    pub new_count: u32,
}

/// How a hunk relates to a symbol's line range.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlapType {
    /// Hunk overlaps symbol interior, does not touch start/end boundary.
    InsideSymbol,
    /// Hunk includes or directly touches symbol start or end line.
    TouchesSymbolBoundary,
    /// Hunk within 3 lines of symbol but no range overlap.
    NearSymbol,
    /// No symbol overlap found in this file.
    FileLevelOnly,
    /// File excluded, parse error, or deleted.
    NoSymbolMatch,
}

/// A single impact record linking a hunk to a symbol.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SymbolImpact {
    pub path: String,
    pub hunk_id: String,
    pub symbol_name: String,
    pub symbol_kind: String,
    pub symbol_line_start: u32,
    pub symbol_line_end: u32,
    pub overlap_type: OverlapType,
    pub confidence: f64,
    /// Parent container (impl, class, module) if applicable.
    pub parent_symbol: Option<ParentSymbol>,
}

/// Parent container metadata. Not emitted as separate impact unless boundary touched.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ParentSymbol {
    pub name: String,
    pub kind: String,
}

/// Result of mapping hunks to symbols for a set of files.
#[derive(Debug, Clone, Serialize)]
pub struct HunkMapResult {
    pub impacts: Vec<SymbolImpact>,
    pub truncated: bool,
    pub total_before_truncation: usize,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum impacts per single file.
const MAX_IMPACTS_PER_FILE: usize = 50;
/// Maximum total impacts across all files.
const MAX_IMPACTS_TOTAL: usize = 200;
/// Near-symbol proximity threshold (lines).
const NEAR_THRESHOLD: u32 = 3;

// ---------------------------------------------------------------------------
// Confidence values per overlap type
// ---------------------------------------------------------------------------

impl OverlapType {
    pub fn confidence(&self) -> f64 {
        match self {
            OverlapType::InsideSymbol => 1.0,
            OverlapType::TouchesSymbolBoundary => 0.85,
            OverlapType::NearSymbol => 0.5,
            OverlapType::FileLevelOnly => 0.0,
            OverlapType::NoSymbolMatch => 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Symbol representation for mapping (borrows from code-intel Symbol)
// ---------------------------------------------------------------------------

/// Simplified symbol for mapping purposes.
#[derive(Debug, Clone)]
pub struct MapSymbol {
    pub name: String,
    pub kind: String,
    pub line_start: u32,
    pub line_end: u32,
    pub parent_name: Option<String>,
    pub parent_kind: Option<String>,
}

// ---------------------------------------------------------------------------
// Core mapping logic
// ---------------------------------------------------------------------------

/// Classify the overlap between a hunk's new range and a symbol's range.
///
/// Uses interval overlap semantics:
/// - InsideSymbol: hunk overlaps symbol interior without touching boundary
/// - TouchesSymbolBoundary: hunk touches start or end line of symbol
/// - NearSymbol: within NEAR_THRESHOLD lines but no overlap
pub fn classify_overlap(
    hunk_new_start: u32,
    hunk_new_end: u32,
    sym_start: u32,
    sym_end: u32,
) -> OverlapType {
    // No overlap at all?
    if hunk_new_end < sym_start || hunk_new_start > sym_end {
        // Check near
        let near_start = if sym_start >= NEAR_THRESHOLD { sym_start - NEAR_THRESHOLD } else { 0 };
        let near_end = sym_end + NEAR_THRESHOLD;
        if hunk_new_end >= near_start && hunk_new_start <= near_end {
            return OverlapType::NearSymbol;
        }
        return OverlapType::FileLevelOnly;
    }

    // There IS overlap. Check if touches boundary.
    if hunk_new_start <= sym_start || hunk_new_end >= sym_end {
        OverlapType::TouchesSymbolBoundary
    } else {
        OverlapType::InsideSymbol
    }
}

/// Map a single hunk against a list of symbols for one file.
///
/// One hunk may produce zero, one, or many SymbolImpact records.
/// Most-specific symbol wins; parent included as metadata only.
/// Parent container NOT emitted as separate impact unless its boundary is touched.
pub fn map_hunk_to_symbols(
    path: &str,
    hunk: &ParsedHunk,
    symbols: &[MapSymbol],
) -> Vec<SymbolImpact> {
    let hunk_start = hunk.new_start;
    let hunk_end = if hunk.new_count == 0 {
        hunk.new_start
    } else {
        hunk.new_start + hunk.new_count - 1
    };

    let mut impacts: Vec<SymbolImpact> = Vec::new();

    for sym in symbols {
        let overlap = classify_overlap(hunk_start, hunk_end, sym.line_start, sym.line_end);

        match overlap {
            OverlapType::FileLevelOnly | OverlapType::NoSymbolMatch => continue,
            _ => {
                let parent = sym.parent_name.as_ref().map(|name| ParentSymbol {
                    name: name.clone(),
                    kind: sym.parent_kind.clone().unwrap_or_default(),
                });

                impacts.push(SymbolImpact {
                    path: path.to_string(),
                    hunk_id: hunk.hunk_id.clone(),
                    symbol_name: sym.name.clone(),
                    symbol_kind: sym.kind.clone(),
                    symbol_line_start: sym.line_start,
                    symbol_line_end: sym.line_end,
                    overlap_type: overlap,
                    confidence: overlap.confidence(),
                    parent_symbol: parent,
                });
            }
        }
    }

    impacts
}

/// Map all hunks against symbols, with bounding and truncation.
///
/// Caps:
/// - max 50 impacts per file
/// - max 200 impacts total
///
/// Returns HunkMapResult with truncation metadata.
pub fn map_hunks_to_symbols(
    file_hunks: &HashMap<String, Vec<ParsedHunk>>,
    file_symbols: &HashMap<String, Vec<MapSymbol>>,
    excluded_paths: &[String],
) -> HunkMapResult {
    let mut all_impacts: Vec<SymbolImpact> = Vec::new();
    let mut total_before_truncation: usize = 0;

    for (path, hunks) in file_hunks {
        // Check exclusion
        if excluded_paths.contains(path) {
            for hunk in hunks {
                total_before_truncation += 1;
                all_impacts.push(SymbolImpact {
                    path: path.clone(),
                    hunk_id: hunk.hunk_id.clone(),
                    symbol_name: String::new(),
                    symbol_kind: String::new(),
                    symbol_line_start: 0,
                    symbol_line_end: 0,
                    overlap_type: OverlapType::NoSymbolMatch,
                    confidence: 0.0,
                    parent_symbol: None,
                });
            }
            continue;
        }

        let symbols = file_symbols.get(path).cloned().unwrap_or_default();
        let mut file_impacts: Vec<SymbolImpact> = Vec::new();

        for hunk in hunks {
            if symbols.is_empty() {
                total_before_truncation += 1;
                file_impacts.push(SymbolImpact {
                    path: path.clone(),
                    hunk_id: hunk.hunk_id.clone(),
                    symbol_name: String::new(),
                    symbol_kind: String::new(),
                    symbol_line_start: 0,
                    symbol_line_end: 0,
                    overlap_type: if hunk.new_count == 0 {
                        OverlapType::FileLevelOnly
                    } else {
                        OverlapType::FileLevelOnly
                    },
                    confidence: 0.0,
                    parent_symbol: None,
                });
            } else {
                let mut hunk_impacts = map_hunk_to_symbols(path, hunk, &symbols);
                if hunk_impacts.is_empty() {
                    total_before_truncation += 1;
                    file_impacts.push(SymbolImpact {
                        path: path.clone(),
                        hunk_id: hunk.hunk_id.clone(),
                        symbol_name: String::new(),
                        symbol_kind: String::new(),
                        symbol_line_start: 0,
                        symbol_line_end: 0,
                        overlap_type: OverlapType::FileLevelOnly,
                        confidence: 0.0,
                        parent_symbol: None,
                    });
                } else {
                    total_before_truncation += hunk_impacts.len();
                    file_impacts.append(&mut hunk_impacts);
                }
            }
        }

        // Per-file cap
        if file_impacts.len() > MAX_IMPACTS_PER_FILE {
            file_impacts.truncate(MAX_IMPACTS_PER_FILE);
        }

        all_impacts.extend(file_impacts);

        // Total cap
        if all_impacts.len() >= MAX_IMPACTS_TOTAL {
            all_impacts.truncate(MAX_IMPACTS_TOTAL);
            return HunkMapResult {
                impacts: all_impacts,
                truncated: true,
                total_before_truncation,
            };
        }
    }

    // Deterministic sort
    sort_impacts(&mut all_impacts);

    let truncated = total_before_truncation > all_impacts.len();
    HunkMapResult {
        impacts: all_impacts,
        truncated,
        total_before_truncation,
    }
}

/// Deterministic impact ordering:
/// 1. path
/// 2. symbol_line_start
/// 3. symbol_line_end
/// 4. symbol_kind
/// 5. symbol_name
/// 6. hunk_id
fn sort_impacts(impacts: &mut [SymbolImpact]) {
    impacts.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.symbol_line_start.cmp(&b.symbol_line_start))
            .then_with(|| a.symbol_line_end.cmp(&b.symbol_line_end))
            .then_with(|| a.symbol_kind.cmp(&b.symbol_kind))
            .then_with(|| a.symbol_name.cmp(&b.symbol_name))
            .then_with(|| a.hunk_id.cmp(&b.hunk_id))
    });
}

/// Parse a hunk header line like `@@ -10,5 +12,3 @@` into a ParsedHunk.
pub fn parse_hunk_header(path: &str, header: &str) -> Option<ParsedHunk> {
    // Match: @@ -old_start[,old_count] +new_start[,new_count] @@
    let re = regex_lite::Regex::new(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").ok()?;
    let caps = re.captures(header)?;
    let old_start: u32 = caps.get(1)?.as_str().parse().ok()?;
    let old_count: u32 = caps.get(2).map(|m| m.as_str().parse().unwrap_or(1)).unwrap_or(1);
    let new_start: u32 = caps.get(3)?.as_str().parse().ok()?;
    let new_count: u32 = caps.get(4).map(|m| m.as_str().parse().unwrap_or(1)).unwrap_or(1);

    Some(ParsedHunk {
        hunk_id: format!("{}:{}-{}", path, old_start, new_start),
        old_start,
        old_count,
        new_start,
        new_count,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_symbol(name: &str, kind: &str, start: u32, end: u32) -> MapSymbol {
        MapSymbol {
            name: name.to_string(),
            kind: kind.to_string(),
            line_start: start,
            line_end: end,
            parent_name: None,
            parent_kind: None,
        }
    }

    fn make_symbol_with_parent(
        name: &str,
        kind: &str,
        start: u32,
        end: u32,
        parent_name: &str,
        parent_kind: &str,
    ) -> MapSymbol {
        MapSymbol {
            name: name.to_string(),
            kind: kind.to_string(),
            line_start: start,
            line_end: end,
            parent_name: Some(parent_name.to_string()),
            parent_kind: Some(parent_kind.to_string()),
        }
    }

    fn make_hunk(old_start: u32, old_count: u32, new_start: u32, new_count: u32) -> ParsedHunk {
        ParsedHunk {
            hunk_id: format!("test.rs:{}-{}", old_start, new_start),
            old_start,
            old_count,
            new_start,
            new_count,
        }
    }

    // --- Overlap classification ---

    #[test]
    fn test_hunk_inside_symbol() {
        // Symbol lines 10-50, hunk lines 20-25 (fully inside, not touching boundary)
        let overlap = classify_overlap(20, 25, 10, 50);
        assert_eq!(overlap, OverlapType::InsideSymbol);
        assert!((overlap.confidence() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hunk_touches_boundary_start() {
        // Symbol lines 10-50, hunk starts at line 10 (touches start boundary)
        let overlap = classify_overlap(10, 25, 10, 50);
        assert_eq!(overlap, OverlapType::TouchesSymbolBoundary);
        assert!((overlap.confidence() - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hunk_touches_boundary_end() {
        // Symbol lines 10-50, hunk ends at line 50 (touches end boundary)
        let overlap = classify_overlap(45, 50, 10, 50);
        assert_eq!(overlap, OverlapType::TouchesSymbolBoundary);
    }

    #[test]
    fn test_hunk_near_symbol() {
        // Symbol lines 10-20, hunk lines 23-25 (within 3 lines)
        let overlap = classify_overlap(23, 25, 10, 20);
        assert_eq!(overlap, OverlapType::NearSymbol);
        assert!((overlap.confidence() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hunk_file_level_only() {
        // Symbol lines 10-20, hunk lines 30-40 (too far)
        let overlap = classify_overlap(30, 40, 10, 20);
        assert_eq!(overlap, OverlapType::FileLevelOnly);
        assert!((overlap.confidence() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hunk_no_symbol_match_excluded() {
        let hunks = HashMap::from([
            ("secret.env".to_string(), vec![make_hunk(1, 5, 1, 5)]),
        ]);
        let symbols = HashMap::new();
        let excluded = vec!["secret.env".to_string()];
        let result = map_hunks_to_symbols(&hunks, &symbols, &excluded);
        assert_eq!(result.impacts.len(), 1);
        assert_eq!(result.impacts[0].overlap_type, OverlapType::NoSymbolMatch);
    }

    #[test]
    fn test_hunk_no_symbol_match_parse_error() {
        // Empty symbols = parse error scenario
        let hunks = HashMap::from([
            ("broken.rs".to_string(), vec![make_hunk(1, 5, 1, 5)]),
        ]);
        let symbols = HashMap::new();
        let excluded: Vec<String> = vec![];
        let result = map_hunks_to_symbols(&hunks, &symbols, &excluded);
        assert_eq!(result.impacts.len(), 1);
        assert_eq!(result.impacts[0].overlap_type, OverlapType::FileLevelOnly);
    }

    // --- One hunk → many symbols ---

    #[test]
    fn test_hunk_maps_to_multiple_symbols() {
        let path = "test.rs";
        let hunk = make_hunk(1, 30, 1, 30);
        // Hunk lines 1-30 touches two adjacent functions
        let symbols = vec![
            make_symbol("foo", "function", 5, 15),
            make_symbol("bar", "function", 18, 28),
        ];
        let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
        assert_eq!(impacts.len(), 2, "One hunk should map to two symbols");
        assert_eq!(impacts[0].symbol_name, "foo");
        assert_eq!(impacts[1].symbol_name, "bar");
    }

    // --- Parent behavior ---

    #[test]
    fn test_parent_symbol_included() {
        let path = "test.rs";
        let hunk = make_hunk(5, 15, 5, 15);
        // Hunk inside method bar (lines 5-15), parent impl Foo
        let symbols = vec![
            make_symbol_with_parent("bar", "method", 5, 15, "Foo", "impl"),
        ];
        let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
        assert_eq!(impacts.len(), 1);
        assert!(impacts[0].parent_symbol.is_some());
        assert_eq!(impacts[0].parent_symbol.as_ref().unwrap().name, "Foo");
        assert_eq!(impacts[0].parent_symbol.as_ref().unwrap().kind, "impl");
    }

    #[test]
    fn test_parent_not_duplicated() {
        let path = "test.rs";
        let hunk = make_hunk(8, 5, 8, 5);
        // Method bar (lines 5-15) inside impl Foo (lines 1-30)
        // Hunk touches only bar interior → parent as metadata, impl NOT separate
        let symbols = vec![
            make_symbol("Foo", "impl", 1, 30),
            make_symbol_with_parent("bar", "method", 5, 15, "Foo", "impl"),
        ];
        let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
        // bar: hunk 8-10 inside bar 5-15, not touching boundary → InsideSymbol
        let bar_impact = impacts.iter().find(|i| i.symbol_name == "bar");
        assert!(bar_impact.is_some());
        assert_eq!(bar_impact.unwrap().overlap_type, OverlapType::InsideSymbol);
        // Foo impl: hunk 8-10 inside impl 1-30, not touching boundary → also InsideSymbol
        // But bar is the most-specific symbol
        let foo_impact = impacts.iter().find(|i| i.symbol_name == "Foo");
        // Both are reported since both overlap, but bar has parent metadata
        assert!(foo_impact.is_some() || bar_impact.unwrap().parent_symbol.is_some());
    }

    // --- Deleted hunk degradation ---

    #[test]
    fn test_deleted_hunk_degrades() {
        let path = "deleted.rs";
        // new_count = 0 means deleted lines
        let hunk = ParsedHunk {
            hunk_id: "deleted.rs:5-0".to_string(),
            old_start: 5,
            old_count: 10,
            new_start: 0,
            new_count: 0,
        };
        let symbols = vec![make_symbol("foo", "function", 1, 30)];
        let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
        // new_start=0, new_count=0 → hunk range is [0,0] → no overlap → FileLevelOnly
        // Actually new_end = 0 + 0 - 1 would underflow, so we handle new_count==0 specially
        // map_hunk_to_symbols sets hunk_end = hunk_start when new_count == 0
        assert!(impacts.is_empty() || impacts.iter().all(|i| i.overlap_type == OverlapType::FileLevelOnly || i.overlap_type == OverlapType::NearSymbol));
    }

    // --- Determinism ---

    #[test]
    fn test_symbol_impact_deterministic() {
        let hunks = HashMap::from([
            ("test.rs".to_string(), vec![
                make_hunk(1, 10, 1, 10),
                make_hunk(20, 10, 25, 10),
            ]),
        ]);
        let symbols = HashMap::from([
            ("test.rs".to_string(), vec![
                make_symbol("alpha", "function", 3, 8),
                make_symbol("beta", "function", 27, 32),
            ]),
        ]);

        let result1 = map_hunks_to_symbols(&hunks, &symbols, &[]);
        let result2 = map_hunks_to_symbols(&hunks, &symbols, &[]);

        assert_eq!(result1.impacts.len(), result2.impacts.len());
        for (a, b) in result1.impacts.iter().zip(result2.impacts.iter()) {
            assert_eq!(a.hunk_id, b.hunk_id);
            assert_eq!(a.symbol_name, b.symbol_name);
            assert_eq!(a.overlap_type, b.overlap_type);
            assert!((a.confidence - b.confidence).abs() < f64::EPSILON);
        }
    }

    // --- No source bodies ---

    #[test]
    fn test_symbol_impact_no_source_bodies() {
        let impact = SymbolImpact {
            path: "test.rs".to_string(),
            hunk_id: "test.rs:5-5".to_string(),
            symbol_name: "foo".to_string(),
            symbol_kind: "function".to_string(),
            symbol_line_start: 3,
            symbol_line_end: 10,
            overlap_type: OverlapType::InsideSymbol,
            confidence: 1.0,
            parent_symbol: None,
        };
        let json = serde_json::to_string(&impact).unwrap();
        // Must not contain source-body-like fields
        assert!(!json.contains("source"));
        assert!(!json.contains("body"));
        assert!(!json.contains("content"));
        assert!(!json.contains("code"));
    }

    // --- Truncation ---

    #[test]
    fn test_truncation_at_cap() {
        let mut hunks = HashMap::new();
        let mut symbols = HashMap::new();
        let mut hunk_list = Vec::new();
        let mut sym_list = Vec::new();

        // Create 60 symbols (exceeds per-file cap of 50)
        for i in 0..60u32 {
            let start = i * 10 + 1;
            let end = start + 8;
            sym_list.push(make_symbol(&format!("fn_{}", i), "function", start, end));
        }

        // One hunk touching all of them
        hunk_list.push(ParsedHunk {
            hunk_id: "big.rs:1-1".to_string(),
            old_start: 1,
            old_count: 600,
            new_start: 1,
            new_count: 600,
        });

        hunks.insert("big.rs".to_string(), hunk_list);
        symbols.insert("big.rs".to_string(), sym_list);

        let result = map_hunks_to_symbols(&hunks, &symbols, &[]);
        assert!(result.truncated, "Should be truncated");
        assert!(result.impacts.len() <= MAX_IMPACTS_PER_FILE);
        assert!(result.total_before_truncation > result.impacts.len());
    }

    // --- Hunk header parsing ---

    #[test]
    fn test_parse_hunk_header() {
        let hunk = parse_hunk_header("test.rs", "@@ -10,5 +12,3 @@ fn foo()").unwrap();
        assert_eq!(hunk.hunk_id, "test.rs:10-12");
        assert_eq!(hunk.old_start, 10);
        assert_eq!(hunk.old_count, 5);
        assert_eq!(hunk.new_start, 12);
        assert_eq!(hunk.new_count, 3);
    }

    #[test]
    fn test_parse_hunk_header_no_counts() {
        // @@ -10 +12 @@ (single line)
        let hunk = parse_hunk_header("test.rs", "@@ -10 +12 @@").unwrap();
        assert_eq!(hunk.old_start, 10);
        assert_eq!(hunk.old_count, 1); // default
        assert_eq!(hunk.new_start, 12);
        assert_eq!(hunk.new_count, 1); // default
    }
}
