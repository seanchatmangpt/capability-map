use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// File inventory
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileEntry {
    /// Absolute path to the file.
    pub path: PathBuf,
    /// BLAKE3 hex hash of content.
    pub hash: String,
    /// File size in bytes.
    pub size: u64,
    /// Guessed language (file extension or heuristic).
    pub language: String,
}

// ---------------------------------------------------------------------------
// Symbols extracted from source files
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub line: usize,
    pub file_path: PathBuf,
}

// ---------------------------------------------------------------------------
// Capabilities detected in the codebase
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectedCapability {
    pub capability: String,
    pub matched_term: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub classification: Classification,
}

// ---------------------------------------------------------------------------
// Classification taxonomy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Classification {
    Live,
    Partial,
    CapabilitySeed,
    LegacyName,
    Dormant,
    BrokenButReal,
    DocOnly,
    TestOnly,
    Ambiguous,
}

impl std::fmt::Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Classification::Live => "LIVE",
            Classification::Partial => "PARTIAL",
            Classification::CapabilitySeed => "CAPABILITY_SEED",
            Classification::LegacyName => "LEGACY_NAME",
            Classification::Dormant => "DORMANT",
            Classification::BrokenButReal => "BROKEN_BUT_REAL",
            Classification::DocOnly => "DOC_ONLY",
            Classification::TestOnly => "TEST_ONLY",
            Classification::Ambiguous => "AMBIGUOUS",
        };
        write!(f, "{s}")
    }
}

// ---------------------------------------------------------------------------
// Scan receipt
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReceipt {
    pub id: String,
    pub timestamp: String,
    pub schema_version: String,
    pub root_paths: Vec<PathBuf>,
    pub file_count: usize,
    pub total_bytes: u64,
    pub root_hash: String,
    pub entries: Vec<ReceiptEntry>,
    pub system_info: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptEntry {
    pub path: String,
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub pid: u32,
    pub hostname: String,
    pub os: String,
}

// ---------------------------------------------------------------------------
// No-deletion report
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoDeletionReport {
    pub before_id: String,
    pub after_id: String,
    pub pass: bool,
    pub deleted_files: Vec<String>,
    pub added_files: Vec<String>,
    pub modified_files: Vec<String>,
}
