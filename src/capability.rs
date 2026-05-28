use crate::models::DetectedCapability;
use regex::Regex;
pub fn detect_capabilities(path: &str, content: &str) -> Vec<DetectedCapability> {
    let mut caps = Vec::new();
    let keywords = vec!["Genesis", "Construct8", "Pair2"];
    for kw in keywords {
        let pattern = format!(r"(?i)\b{}\b", regex::escape(kw));
        if let Ok(re) = Regex::new(&pattern) {
            if let Some(mat) = re.find(content) {
                caps.push(DetectedCapability {
                    file_path: path.to_string(),
                    capability: kw.to_string(),
                    matched_term: mat.as_str().to_string(),
                    snippet: "".to_string(),
                    classification: "LIVE".to_string(),
                });
            }
        }
    }
    caps
}
