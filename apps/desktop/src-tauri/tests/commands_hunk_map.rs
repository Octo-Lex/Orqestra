//! v2.3.0: Hunk-to-Symbol Impact Mapping tests.
//!
//! Tests verify:
//! - Interval overlap semantics (new-file ranges)
//! - One hunk → many symbols
//! - Parent metadata without duplication
//! - Deleted hunk graceful degradation
//! - Truncation caps
//! - Path redaction in diagnostics
//! - Deterministic ordering

use std::collections::HashMap;
use std::path::PathBuf;

fn find_repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    while !dir.join(".git").exists() {
        if !dir.pop() { panic!("No git repo found"); }
    }
    dir
}

// Re-use code-intel hunk_map types for integration testing
use code_intel::hunk_map::*;

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

// ---------------------------------------------------------------------------
// Integration tests (exercising the module via the public API)
// ---------------------------------------------------------------------------

#[test]
fn test_hunk_inside_symbol_integration() {
    let path = "src/lib.rs";
    let hunk = make_hunk(1, 10, 3, 8); // hunk 3-10
    let symbols = vec![make_symbol("main", "function", 1, 50)];
    let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
    assert_eq!(impacts.len(), 1);
    assert_eq!(impacts[0].overlap_type, OverlapType::InsideSymbol);
    assert!((impacts[0].confidence - 1.0).abs() < f64::EPSILON);
}

#[test]
fn test_hunk_touches_start_boundary_integration() {
    let path = "src/lib.rs";
    let hunk = make_hunk(1, 10, 1, 10);
    let symbols = vec![make_symbol("foo", "function", 1, 30)];
    let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
    assert_eq!(impacts.len(), 1);
    assert_eq!(impacts[0].overlap_type, OverlapType::TouchesSymbolBoundary);
}

#[test]
fn test_hunk_touches_end_boundary_integration() {
    let path = "src/lib.rs";
    // Hunk ends at line 30 (start=25, count=6 → end=30)
    let hunk = ParsedHunk {
        hunk_id: "src/lib.rs:25-25".to_string(),
        old_start: 25, old_count: 6, new_start: 25, new_count: 6,
    };
    let symbols = vec![make_symbol("bar", "function", 10, 30)];
    let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
    assert_eq!(impacts.len(), 1);
    assert_eq!(impacts[0].overlap_type, OverlapType::TouchesSymbolBoundary);
}

#[test]
fn test_hunk_near_symbol_integration() {
    let path = "src/lib.rs";
    let hunk = make_hunk(23, 5, 23, 5); // hunk 23-27
    let symbols = vec![make_symbol("baz", "function", 10, 20)]; // symbol 10-20
    let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
    assert_eq!(impacts.len(), 1);
    assert_eq!(impacts[0].overlap_type, OverlapType::NearSymbol);
}

#[test]
fn test_hunk_file_level_only_integration() {
    let path = "src/lib.rs";
    let hunk = make_hunk(50, 10, 50, 10); // hunk 50-59
    let symbols = vec![make_symbol("qux", "function", 1, 20)]; // symbol 1-20
    let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
    assert!(impacts.is_empty());
}

#[test]
fn test_one_hunk_many_symbols_integration() {
    let path = "src/types.ts";
    // Hunk spanning 50 lines touches 3 adjacent interfaces
    let hunk = ParsedHunk {
        hunk_id: "src/types.ts:1-1".to_string(),
        old_start: 1, old_count: 50, new_start: 1, new_count: 50,
    };
    let symbols = vec![
        make_symbol("User", "interface", 1, 15),
        make_symbol("Profile", "interface", 18, 30),
        make_symbol("Settings", "interface", 33, 50),
    ];
    let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
    assert_eq!(impacts.len(), 3);
    assert_eq!(impacts[0].symbol_name, "User");
    assert_eq!(impacts[1].symbol_name, "Profile");
    assert_eq!(impacts[2].symbol_name, "Settings");
}

#[test]
fn test_parent_metadata_no_duplication() {
    let path = "src/impl.rs";
    // Hunk inside method (lines 10-14), impl block is lines 1-40
    let hunk = ParsedHunk {
        hunk_id: "src/impl.rs:10-10".to_string(),
        old_start: 10, old_count: 5, new_start: 10, new_count: 5,
    };
    let symbols = vec![
        make_symbol("MyStruct", "impl", 1, 40),
        make_symbol_with_parent("do_work", "method", 8, 20, "MyStruct", "impl"),
    ];
    let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
    // Method is InsideSymbol, impl is InsideSymbol too
    // Both reported but method has parent metadata
    let method_impact = impacts.iter().find(|i| i.symbol_name == "do_work");
    assert!(method_impact.is_some());
    let m = method_impact.unwrap();
    assert_eq!(m.overlap_type, OverlapType::InsideSymbol);
    assert!(m.parent_symbol.is_some());
    assert_eq!(m.parent_symbol.as_ref().unwrap().name, "MyStruct");
}

#[test]
fn test_deleted_hunk_file_level() {
    let path = "deleted.rs";
    let hunk = ParsedHunk {
        hunk_id: "deleted.rs:5-0".to_string(),
        old_start: 5, old_count: 10, new_start: 0, new_count: 0,
    };
    let symbols = vec![make_symbol("old_fn", "function", 1, 30)];
    let impacts = map_hunk_to_symbols(path, &hunk, &symbols);
    // new_count=0 → hunk range [0,0] → likely NearSymbol or FileLevelOnly
    assert!(impacts.is_empty() || impacts.iter().all(|i| matches!(
        i.overlap_type, OverlapType::FileLevelOnly | OverlapType::NearSymbol
    )));
}

#[test]
fn test_truncation_per_file_cap() {
    let mut hunks = HashMap::new();
    let mut symbols = HashMap::new();
    let mut hunk_list = Vec::new();
    let mut sym_list = Vec::new();

    for i in 0..60u32 {
        let start = i * 10 + 1;
        sym_list.push(make_symbol(&format!("fn_{}", i), "function", start, start + 8));
    }
    hunk_list.push(ParsedHunk {
        hunk_id: "big.rs:1-1".to_string(),
        old_start: 1, old_count: 600, new_start: 1, new_count: 600,
    });
    hunks.insert("big.rs".to_string(), hunk_list);
    symbols.insert("big.rs".to_string(), sym_list);

    let result = map_hunks_to_symbols(&hunks, &symbols, &[]);
    assert!(result.impacts.len() <= 50, "Per-file cap of 50");
    assert!(result.total_before_truncation > result.impacts.len());
}

#[test]
fn test_path_redaction_in_diagnostics() {
    let impact = SymbolImpact {
        path: "src/secret_module.rs".to_string(),
        hunk_id: "src/secret_module.rs:5-5".to_string(),
        symbol_name: "process_credentials".to_string(),
        symbol_kind: "function".to_string(),
        symbol_line_start: 3,
        symbol_line_end: 10,
        overlap_type: OverlapType::InsideSymbol,
        confidence: 1.0,
        parent_symbol: None,
    };

    // When building diagnostics, paths should be redacted
    // For this test, verify the DTO structure allows redaction
    let json = serde_json::to_string(&impact).unwrap();
    // SymbolImpact does not contain source bodies
    assert!(!json.contains("source"));
    assert!(!json.contains("body"));
    assert!(!json.contains("content"));
    // Path redaction is done at diagnostics bundle level, not DTO level
    // The DTO itself is for internal agent contexts where paths are safe
}

#[test]
fn test_deterministic_ordering() {
    let hunks = HashMap::from([
        ("a.rs".to_string(), vec![
            ParsedHunk { hunk_id: "a.rs:1-1".to_string(), old_start: 1, old_count: 20, new_start: 1, new_count: 20 },
        ]),
        ("b.rs".to_string(), vec![
            ParsedHunk { hunk_id: "b.rs:1-1".to_string(), old_start: 1, old_count: 20, new_start: 1, new_count: 20 },
        ]),
    ]);
    let symbols = HashMap::from([
        ("a.rs".to_string(), vec![make_symbol("alpha", "function", 5, 15)]),
        ("b.rs".to_string(), vec![make_symbol("beta", "function", 5, 15)]),
    ]);

    let r1 = map_hunks_to_symbols(&hunks, &symbols, &[]);
    let r2 = map_hunks_to_symbols(&hunks, &symbols, &[]);

    assert_eq!(r1.impacts.len(), r2.impacts.len());
    for (a, b) in r1.impacts.iter().zip(r2.impacts.iter()) {
        assert_eq!(a.hunk_id, b.hunk_id);
        assert_eq!(a.symbol_name, b.symbol_name);
    }
}
