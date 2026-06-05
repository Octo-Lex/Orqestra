//! Symbol extraction for Rust and TypeScript.
//!
//! Uses tree-sitter queries to extract symbols. Returns sorted Vec<Symbol>.
//! Content-safe: never includes source bodies.

use crate::{ParseStatus, Symbol, SymbolKind};

/// Maximum ERROR/MISSING node ratio before marking as parse error.
const MAX_ERROR_RATIO: f64 = 0.3;

// ---------------------------------------------------------------------------
// Rust extraction
// ---------------------------------------------------------------------------

pub fn extract_rust(source: &str) -> (Vec<Symbol>, ParseStatus) {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())
        .expect("Failed to load Rust grammar");

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return (vec![], ParseStatus::ParseError),
    };

    let root = tree.root_node();

    if error_ratio(&root) > MAX_ERROR_RATIO {
        return (vec![], ParseStatus::ParseError);
    }

    let mut symbols = Vec::new();
    extract_rust_node(&root, source, None, &mut symbols);

    let status = if root.has_error() {
        ParseStatus::ParseError
    } else {
        ParseStatus::Success
    };

    (symbols, status)
}

fn extract_rust_node(
    node: &tree_sitter::Node,
    source: &str,
    parent: Option<&str>,
    symbols: &mut Vec<Symbol>,
) {
    // Skip ERROR and MISSING nodes
    if node.kind() == "ERROR" || node.is_missing() {
        return;
    }

    match node.kind() {
        "function_item" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                let is_public = has_visibility_pub(node);
                symbols.push(Symbol {
                    name,
                    kind: if parent.is_some() { SymbolKind::Method } else { SymbolKind::Function },
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public,
                    parent: parent.map(String::from),
                });
            }
        }
        "struct_item" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Struct,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_visibility_pub(node),
                    parent: parent.map(String::from),
                });
            }
        }
        "enum_item" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Enum,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_visibility_pub(node),
                    parent: parent.map(String::from),
                });
            }
        }
        "trait_item" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Trait,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_visibility_pub(node),
                    parent: parent.map(String::from),
                });
            }
        }
        "impl_item" => {
            // tree-sitter-rust uses "type" field for impl name, not "name"
            let name = safe_child_field_text(node, "type", source)
                .or_else(|| safe_child_field_text(node, "name", source));
            if let Some(ref impl_name) = name {
                symbols.push(Symbol {
                    name: impl_name.clone(),
                    kind: SymbolKind::Impl,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: true,
                    parent: parent.map(String::from),
                });
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                extract_rust_node(&child, source, name.as_deref(), symbols);
            }
            return;
        }
        "type_item" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::TypeAlias,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_visibility_pub(node),
                    parent: parent.map(String::from),
                });
            }
        }
        "const_item" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Constant,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_visibility_pub(node),
                    parent: parent.map(String::from),
                });
            }
        }
        "mod_item" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Module,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_visibility_pub(node),
                    parent: parent.map(String::from),
                });
            }
        }
        "use_declaration" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "use_clause" || child.kind() == "identifier"
                    || child.kind() == "scoped_identifier" || child.kind() == "use_list"
                {
                    let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                    let last = text.rsplit("::").next().unwrap_or(text).trim().to_string();
                    if !last.is_empty() && last != "self" && last != "super" && last != "crate" {
                        symbols.push(Symbol {
                            name: last,
                            kind: SymbolKind::Import,
                            line_start: node.start_position().row as u32 + 1,
                            line_end: node.end_position().row as u32 + 1,
                            is_public: false,
                            parent: None,
                        });
                    }
                }
            }
        }
        _ => {}
    }

    if node.kind() != "impl_item" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            extract_rust_node(&child, source, parent, symbols);
        }
    }
}

fn has_visibility_pub(node: &tree_sitter::Node) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "visibility_modifier" {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// TypeScript/TSX extraction
// ---------------------------------------------------------------------------

pub fn extract_typescript(source: &str) -> (Vec<Symbol>, ParseStatus) {
    let mut parser = tree_sitter::Parser::new();

    let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
    parser.set_language(&language.into())
        .expect("Failed to load TypeScript grammar");

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => return (vec![], ParseStatus::ParseError),
    };

    let root = tree.root_node();

    if error_ratio(&root) > MAX_ERROR_RATIO {
        return (vec![], ParseStatus::ParseError);
    }

    let mut symbols = Vec::new();
    extract_ts_node(&root, source, None, &mut symbols);

    let status = if root.has_error() {
        ParseStatus::ParseError
    } else {
        ParseStatus::Success
    };

    (symbols, status)
}

fn extract_ts_node(
    node: &tree_sitter::Node,
    source: &str,
    parent: Option<&str>,
    symbols: &mut Vec<Symbol>,
) {
    if node.kind() == "ERROR" || node.is_missing() {
        return;
    }

    match node.kind() {
        "function_declaration" | "generator_function_declaration" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                let is_public = has_export_ancestor(node);
                symbols.push(Symbol {
                    name,
                    kind: if parent.is_some() { SymbolKind::Method } else { SymbolKind::Function },
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public,
                    parent: parent.map(String::from),
                });
            }
        }
        "lexical_declaration" | "variable_declaration" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "variable_declarator" {
                    let name_node = child.child_by_field_name("name");
                    let value_node = child.child_by_field_name("value");
                    if let (Some(n), Some(v)) = (name_node, value_node) {
                        let name = n.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                        let value_kind = v.kind();
                        if value_kind == "arrow_function" || value_kind == "function_expression" {
                            symbols.push(Symbol {
                                name,
                                kind: SymbolKind::Function,
                                line_start: node.start_position().row as u32 + 1,
                                line_end: node.end_position().row as u32 + 1,
                                is_public: has_export_ancestor(node),
                                parent: parent.map(String::from),
                            });
                        }
                    }
                }
            }
        }
        "class_declaration" => {
            let name = safe_child_field_text(node, "name", source);
            if let Some(ref class_name) = name {
                symbols.push(Symbol {
                    name: class_name.clone(),
                    kind: SymbolKind::Class,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_export_ancestor(node),
                    parent: parent.map(String::from),
                });
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                extract_ts_node(&child, source, name.as_deref(), symbols);
            }
            return;
        }
        "interface_declaration" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Interface,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_export_ancestor(node),
                    parent: parent.map(String::from),
                });
            }
        }
        "type_alias_declaration" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::TypeAlias,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_export_ancestor(node),
                    parent: parent.map(String::from),
                });
            }
        }
        "enum_declaration" => {
            if let Some(name) = safe_child_field_text(node, "name", source) {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Enum,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: has_export_ancestor(node),
                    parent: parent.map(String::from),
                });
            }
        }
        "method_definition" | "public_field_definition" => {
            let name = node.child_by_field_name("name")
                .map(|n| n.utf8_text(source.as_bytes()).unwrap_or("").to_string())
                .unwrap_or_default();
            if !name.is_empty() {
                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Method,
                    line_start: node.start_position().row as u32 + 1,
                    line_end: node.end_position().row as u32 + 1,
                    is_public: true,
                    parent: parent.map(String::from),
                });
            }
        }
        "import_statement" => {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            if let Some(start) = text.find('{') {
                if let Some(end) = text.find('}') {
                    let imports = &text[start + 1..end];
                    for name in imports.split(',') {
                        let clean = name.trim()
                            .split(" as ").next().unwrap_or("").trim()
                            .to_string();
                        if !clean.is_empty() {
                            symbols.push(Symbol {
                                name: clean,
                                kind: SymbolKind::Import,
                                line_start: node.start_position().row as u32 + 1,
                                line_end: node.end_position().row as u32 + 1,
                                is_public: false,
                                parent: None,
                            });
                        }
                    }
                }
            }
        }
        _ => {}
    }

    if node.kind() != "class_declaration" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            extract_ts_node(&child, source, parent, symbols);
        }
    }
}

fn has_export_ancestor(node: &tree_sitter::Node) -> bool {
    let mut current = node.parent();
    while let Some(p) = current {
        if p.kind() == "export_statement" {
            return true;
        }
        current = p.parent();
    }
    false
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Safe child field text extraction — returns None if field doesn't exist
/// or if the text extraction panics (defensive against tree-sitter edge cases).
fn safe_child_field_text(node: &tree_sitter::Node, field: &str, source: &str) -> Option<String> {
    let child = node.child_by_field_name(field)?;
    let text = child.utf8_text(source.as_bytes()).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_string())
}

fn error_ratio(root: &tree_sitter::Node) -> f64 {
    let total = root.child_count();
    if total == 0 {
        return 0.0;
    }
    let errors = count_errors(root);
    errors as f64 / total as f64
}

fn count_errors(node: &tree_sitter::Node) -> usize {
    let mut count = 0;
    if node.kind() == "ERROR" || node.is_missing() {
        count += 1;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        count += count_errors(&child);
    }
    count
}
