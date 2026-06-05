# Code Intelligence (v1.8.0+)

## Overview

Code intelligence provides tree-sitter-based symbol extraction for Rust and TypeScript source files. It is used by the bugfix agent for symbol-aware context and by the architect agent for affected symbol analysis.

## Architecture

```
crates/code-intel/     Pure Rust crate (zero Tauri/git-bridge dependency)
├── src/lib.rs         DTOs, public API, language detection, deterministic sorting
├── src/extract.rs     tree-sitter Rust/TS extraction with safe field access
└── src/exclude.rs     File exclusion rules
```

### Circular Dependency Guard

`code-intel` has **zero dependency** on Tauri or `git-bridge`. The dependency direction is:
- `git-bridge` may call `code-intel` ✅
- `code-intel` must not call `git-bridge` or Tauri ❌

## Supported Languages

| Language | tree-sitter Grammar | Notes |
|----------|-------------------|-------|
| Rust | `tree-sitter-rust` 0.24 | Full symbol extraction |
| TypeScript | `tree-sitter-typescript` 0.24 | Full symbol extraction |

## File Exclusion

Files are excluded from symbol extraction if:

| Rule | Criteria |
|------|----------|
| Binary | Extension indicates binary (`.exe`, `.dll`, `.so`, `.png`, etc.) |
| Secret | Filename contains secret patterns (`.env`, `credentials`, `*.pem`, `*.key`) |
| Generated | Path contains `generated`, `target/`, `node_modules/`, `dist/` |
| Too large | File size > 256 KiB |

## Symbol DTOs

```rust
pub struct SymbolSummary {
    pub file: String,
    pub language: CodeLanguage,
    pub parse_status: ParseStatus,
    pub symbols: Vec<Symbol>,
    pub parse_time_ms: u64,
}

pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line_start: u32,
    pub line_end: u32,
    pub parent: Option<String>,
}

pub enum ParseStatus {
    Success,
    ParseError,
    Excluded,
    TooLarge,
    Binary,
    UnknownLanguage,
}

pub enum SymbolKind {
    Function, Struct, Enum, Trait, Impl, Module,
    Interface, Class, Type, Method, Variable,
}
```

## Deterministic Ordering

Symbols are sorted by: `line_start → line_end → kind → name → parent`.

This ensures consistent output across runs and platforms.

## Parse Error Threshold

If the ratio of ERROR/MISSING nodes exceeds **30%**, the parse status is `ParseError`. Malformed but partially parseable code is still processed — the threshold catches truly broken files.

## Agent Integration

### Bugfix Agent

Receives symbol summaries for changed files.

v2.3.0: Also receives hunk-level symbol impacts (high-confidence only: InsideSymbol + TouchesSymbolBoundary).

### Architect Agent

Uses `affected_symbols` in plan output — symbol names, kinds, files, and public visibility.

v2.3.0: Also receives `ArchitectSymbolSummary` with bounded hunk-level impact counts (max 20 individual impacts).

### Docs Agent

Symbol context is **disabled by default** for docs-agent. Only bugfix-agent receives symbol summaries by default.

## Hunk-to-Symbol Impact Mapping (v2.3.0)

v2.3.0 added precision mapping from safe diff hunks to affected symbols.

### Architecture

```
crates/code-intel/
├── src/hunk_map.rs     ParsedHunk, SymbolImpact, OverlapType, interval overlap mapping
```

### Mapping basis

**Primary mapping uses new-file line ranges (`new_start`/`new_count`) against current-file symbols.**
Old ranges are retained for diagnostics and future preimage mapping.

Deleted hunks/files degrade to `FileLevelOnly` or `NoSymbolMatch` without parsing old blobs.

### Overlap types

| Type | Condition | Confidence |
|------|-----------|------------|
| `inside_symbol` | Hunk overlaps symbol interior without touching boundary | 1.0 |
| `touches_symbol_boundary` | Hunk touches symbol start or end line | 0.85 |
| `near_symbol` | Within 3 lines, no overlap | 0.5 |
| `file_level_only` | No symbol overlap found | 0.0 |
| `no_symbol_match` | File excluded, parse error, or deleted | 0.0 |

### One hunk → many symbols

A single hunk may map to zero, one, or many symbols. All impacts sorted deterministically.

### Parent/container behavior

- Most-specific symbol wins
- Parent included as metadata (`parent_symbol` field)
- Parent container NOT emitted as separate impact unless its boundary is directly touched

### Bounded output

- Max 50 impacts per file
- Max 200 impacts total
- Truncation reported via `truncated` flag

### Path redaction

Internal agent contexts use full paths.
Diagnostics and support bundles redact or omit raw paths unless explicitly classified safe.

## Tauri Commands

- `extract_symbols_cmd(file_path, source)` — single file extraction
- `extract_symbols_batch_cmd(project_root, files)` — batch extraction

## Test Coverage

29 code intelligence tests + 16 hunk_map tests + 11 integration tests verify:
- Rust and TypeScript extraction
- File exclusion rules (binary, secret, generated, too large)
- Parse error threshold
- Deterministic ordering
- Bounded parsing
- Symbol DTO structure
- Interval overlap semantics (all 5 overlap types)
- One hunk → many symbols
- Parent metadata without duplication
- Deleted hunk graceful degradation
- Truncation caps (per-file and total)
- Path redaction in diagnostics
- Deterministic impact ordering
