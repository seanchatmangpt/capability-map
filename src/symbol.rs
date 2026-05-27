use crate::models::Symbol;
use std::path::Path;

// Known symbol-definition patterns per language.
// Each tuple is (kind_label, regex_pattern).
static PATTERNS: &[(&str, &str)] = &[
    ("fn",     r"(?m)^\s*pub\s+(?:async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)"),
    ("fn",     r"(?m)^\s*(?:async\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)"),
    ("struct", r"(?m)^\s*pub\s+struct\s+([A-Z][a-zA-Z0-9_]*)"),
    ("struct", r"(?m)^\s*struct\s+([A-Z][a-zA-Z0-9_]*)"),
    ("enum",   r"(?m)^\s*pub\s+enum\s+([A-Z][a-zA-Z0-9_]*)"),
    ("enum",   r"(?m)^\s*enum\s+([A-Z][a-zA-Z0-9_]*)"),
    ("trait",  r"(?m)^\s*pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)"),
    ("trait",  r"(?m)^\s*trait\s+([A-Z][a-zA-Z0-9_]*)"),
    ("impl",   r"(?m)^\s*impl(?:<[^>]+>)?\s+([A-Z][a-zA-Z0-9_<>]*)"),
    ("def",    r"(?m)^\s*def\s+([a-zA-Z_][a-zA-Z0-9_]*)"),
    ("class",  r"(?m)^\s*class\s+([A-Z][a-zA-Z0-9_]*)"),
    ("func",   r"(?m)^\s*func\s+([a-zA-Z_][a-zA-Z0-9_]*)"),
    ("type",   r"(?m)^\s*type\s+([A-Z][a-zA-Z0-9_]*)"),
];

pub fn extract_symbols(path: &Path, content: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (kind, pat) in PATTERNS {
        let Ok(re) = regex::Regex::new(pat) else {
            continue;
        };
        for cap in re.captures_iter(content) {
            let name = cap.get(1).map_or("", |m| m.as_str()).to_string();
            if name.is_empty() {
                continue;
            }
            // Approximate line number by counting '\n' before the match start.
            let start = cap.get(0).map_or(0, |m| m.start());
            let line = content[..start].chars().filter(|&c| c == '\n').count() + 1;
            symbols.push(Symbol {
                name,
                kind: kind.to_string(),
                line,
                file_path: path.to_path_buf(),
            });
        }
    }

    symbols.dedup_by(|a, b| a.name == b.name && a.kind == b.kind && a.line == b.line);
    symbols
}

/// Returns only symbols from `all` whose file path matches `path`.
pub fn symbols_for_file<'a>(all: &'a [Symbol], path: &Path) -> Vec<&'a Symbol> {
    all.iter().filter(|s| s.file_path == path).collect()
}
