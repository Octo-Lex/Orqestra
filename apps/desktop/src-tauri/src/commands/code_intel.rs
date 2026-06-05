//! Code intelligence commands — tree-sitter-based symbol extraction.
//!
//! v1.8.0: Read-only symbol extraction for Rust and TypeScript files.
//! Content-safe: returns symbol names/kinds/ranges, never source bodies.

use tauri::command;

/// Extract symbols from a single source file.
#[command]
pub fn extract_symbols_cmd(
    path: String,
    source: String,
) -> Result<String, String> {
    let result = code_intel::extract_symbols(&path, &source);
    serde_json::to_string(&result)
        .map_err(|e| format!("Failed to serialize symbols: {e}"))
}

/// Extract symbols from multiple files.
/// Returns array of SymbolSummary.
#[command]
pub fn extract_symbols_batch_cmd(
    files: Vec<serde_json::Value>,
) -> Result<String, String> {
    let results: Vec<code_intel::SymbolSummary> = files.iter()
        .filter_map(|f| {
            let path = f.get("path")?.as_str()?;
            let source = f.get("source")?.as_str()?;
            Some(code_intel::extract_symbols(path, source))
        })
        .collect();

    serde_json::to_string(&results)
        .map_err(|e| format!("Failed to serialize batch: {e}"))
}
