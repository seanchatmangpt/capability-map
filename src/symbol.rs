use crate::models::Symbol;
use regex::Regex;
pub fn extract_symbols(path: &str, content: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    let patterns = [(r"fn\s+([a-zA-Z_][a-zA-Z0-9_]*)", "function"), (r"struct\s+([a-zA-Z_][a-zA-Z0-9_]*)", "struct")];
    for (pat, kind) in patterns {
        if let Ok(re) = Regex::new(pat) {
            for cap in re.captures_iter(content) {
                symbols.push(Symbol { file_path: path.to_string(), name: cap[1].to_string(), kind: kind.to_string() });
            }
        }
    }
    symbols
}
