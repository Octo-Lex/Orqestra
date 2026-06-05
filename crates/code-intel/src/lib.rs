//! Code Intelligence — tree-sitter-based symbol extraction.
//!
//! Pure Rust crate. Zero Tauri dependency. Zero git-bridge dependency.
//!
//! Extracts function, type, class, interface, import, and export symbols
//! from Rust and TypeScript/TSX source files. Content-safe: outputs names,
//! kinds, line ranges, and visibility — never source bodies.

mod extract;
mod exclude;
pub mod hunk_map;

use serde::Serialize;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// Language detected by file extension.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CodeLanguage {
    Rust,
    TypeScript,
    Tsx,
    Unknown,
}

/// Why a file was not parsed.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ParseStatus {
    Success,
    ParseError,
    Excluded,
    TooLarge,
    Binary,
    Secret,
}

/// A single extracted symbol.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line_start: u32,
    pub line_end: u32,
    pub is_public: bool,
    pub parent: Option<String>,
}

/// Kind of extracted symbol.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    TypeAlias,
    Interface,
    Class,
    Module,
    Import,
    Constant,
    Variable,
}

/// Summary of symbols extracted from a single file.
#[derive(Debug, Clone, Serialize)]
pub struct SymbolSummary {
    pub path: String,
    pub language: CodeLanguage,
    pub symbols: Vec<Symbol>,
    pub parse_status: ParseStatus,
    pub parse_latency_ms: u64,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Extract symbols from source code at the given path.
///
/// Returns a SymbolSummary with:
/// - Language detected by extension
/// - Sorted symbols (by line_start, kind, name, parent)
/// - Parse status (Success, ParseError, Excluded, TooLarge, Binary, Secret)
///
/// Content-safe: never includes source bodies in output.
/// Read-only: never writes repository files.
pub fn extract_symbols(
    path: &str,
    source: &str,
) -> SymbolSummary {
    let start = std::time::Instant::now();

    let language = detect_language(path);

    // Check exclusions
    if let Some(status) = exclude::check_excluded(path, source) {
        return SymbolSummary {
            path: path.to_string(),
            language,
            symbols: vec![],
            parse_status: status,
            parse_latency_ms: start.elapsed().as_millis() as u64,
        };
    }

    let (symbols, parse_status) = match language {
        CodeLanguage::Rust => extract::extract_rust(source),
        CodeLanguage::TypeScript | CodeLanguage::Tsx => extract::extract_typescript(source),
        CodeLanguage::Unknown => (vec![], ParseStatus::Excluded),
    };

    // Deterministic ordering
    let mut sorted = symbols;
    sort_symbols(&mut sorted);

    SymbolSummary {
        path: path.to_string(),
        language,
        symbols: sorted,
        parse_status,
        parse_latency_ms: start.elapsed().as_millis() as u64,
    }
}

/// Detect language from file extension.
pub fn detect_language(path: &str) -> CodeLanguage {
    let lower = path.to_lowercase();
    if lower.ends_with(".rs") {
        CodeLanguage::Rust
    } else if lower.ends_with(".tsx") || lower.ends_with(".jsx") {
        CodeLanguage::Tsx
    } else if lower.ends_with(".ts") || lower.ends_with(".js") {
        CodeLanguage::TypeScript
    } else {
        CodeLanguage::Unknown
    }
}

/// Deterministic symbol ordering:
/// 1. line_start
/// 2. line_end
/// 3. kind (variant index)
/// 4. name
/// 5. parent
fn sort_symbols(symbols: &mut [Symbol]) {
    symbols.sort_by(|a, b| {
        a.line_start
            .cmp(&b.line_start)
            .then_with(|| a.line_end.cmp(&b.line_end))
            .then_with(|| kind_order(&a.kind).cmp(&kind_order(&b.kind)))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.parent.cmp(&b.parent))
    });
}

fn kind_order(kind: &SymbolKind) -> u8 {
    match kind {
        SymbolKind::Module => 0,
        SymbolKind::Import => 1,
        SymbolKind::Constant => 2,
        SymbolKind::Struct => 3,
        SymbolKind::Enum => 4,
        SymbolKind::Trait => 5,
        SymbolKind::Impl => 6,
        SymbolKind::Interface => 7,
        SymbolKind::Class => 8,
        SymbolKind::TypeAlias => 9,
        SymbolKind::Function => 10,
        SymbolKind::Method => 11,
        SymbolKind::Variable => 12,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_language_rust() {
        assert_eq!(detect_language("src/main.rs"), CodeLanguage::Rust);
    }

    #[test]
    fn detect_language_typescript() {
        assert_eq!(detect_language("src/app.ts"), CodeLanguage::TypeScript);
    }

    #[test]
    fn detect_language_tsx() {
        assert_eq!(detect_language("src/Component.tsx"), CodeLanguage::Tsx);
    }

    #[test]
    fn detect_language_unknown() {
        assert_eq!(detect_language("README.md"), CodeLanguage::Unknown);
    }

    #[test]
    fn sorting_is_deterministic() {
        let mut symbols = vec![
            Symbol { name: "beta".into(), kind: SymbolKind::Function, line_start: 10, line_end: 15, is_public: true, parent: None },
            Symbol { name: "alpha".into(), kind: SymbolKind::Function, line_start: 5, line_end: 8, is_public: false, parent: None },
            Symbol { name: "gamma".into(), kind: SymbolKind::Struct, line_start: 5, line_end: 6, is_public: true, parent: None },
        ];
        sort_symbols(&mut symbols);
        assert_eq!(symbols[0].name, "gamma"); // same line_start=5, kind Struct < Function
        assert_eq!(symbols[1].name, "alpha"); // line_start=5, kind Function
        assert_eq!(symbols[2].name, "beta"); // line_start=10
    }
}
