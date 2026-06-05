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

Receives symbol summaries for changed files. Symbols are file-level (not hunk-level) — conservative scope.

### Architect Agent

Uses `affected_symbols` in plan output — symbol names, kinds, files, and public visibility.

### Docs Agent

Symbol context is **disabled by default** for docs-agent. Only bugfix-agent receives symbol summaries by default.

## Tauri Commands

- `extract_symbols_cmd(file_path, source)` — single file extraction
- `extract_symbols_batch_cmd(project_root, files)` — batch extraction

## Test Coverage

29 code intelligence tests verify:
- Rust and TypeScript extraction
- File exclusion rules (binary, secret, generated, too large)
- Parse error threshold
- Deterministic ordering
- Bounded parsing
- Symbol DTO structure
