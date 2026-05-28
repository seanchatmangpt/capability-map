use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry { pub path: String, pub hash: String, pub size: i64, pub language: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol { pub file_path: String, pub name: String, pub kind: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedCapability { pub file_path: String, pub capability: String, pub matched_term: String, pub snippet: String, pub classification: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt { pub id: String, pub timestamp: String, pub aggregate_hash: String, pub file_count: i64, pub total_bytes: i64, pub files: Vec<FileEntry> }
