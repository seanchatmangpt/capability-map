use rusqlite::{Connection, params};
use std::path::Path;
use anyhow::{Context, Result};
use crate::models::{DetectedCapability, ScanReceipt, Symbol};

/// Open or create the catalog SQLite database.
pub fn open(db_path: &Path) -> Result<Connection> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(db_path).context("opening workspace.sqlite")?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .context("PRAGMA init")?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS scan_runs (
            id          TEXT PRIMARY KEY,
            started_at  TEXT NOT NULL,
            roots       TEXT NOT NULL,
            file_count  INTEGER DEFAULT 0,
            total_bytes INTEGER DEFAULT 0,
            status      TEXT DEFAULT 'running'
        );
        CREATE TABLE IF NOT EXISTS repositories (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id TEXT NOT NULL,
            root        TEXT NOT NULL,
            name        TEXT NOT NULL,
            ecosystem   TEXT NOT NULL,
            git_head    TEXT
        );
        CREATE TABLE IF NOT EXISTS files (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id TEXT NOT NULL,
            path        TEXT NOT NULL,
            size        INTEGER NOT NULL DEFAULT 0,
            hash        TEXT NOT NULL DEFAULT '',
            language    TEXT NOT NULL DEFAULT '',
            is_test     INTEGER DEFAULT 0,
            is_doc      INTEGER DEFAULT 0,
            is_config   INTEGER DEFAULT 0,
            is_script   INTEGER DEFAULT 0,
            is_binary   INTEGER DEFAULT 0,
            git_root    TEXT
        );
        CREATE TABLE IF NOT EXISTS symbols (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id TEXT NOT NULL,
            name        TEXT NOT NULL,
            kind        TEXT NOT NULL,
            line        INTEGER NOT NULL,
            file_path   TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS dependencies (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id TEXT NOT NULL,
            name        TEXT NOT NULL,
            version     TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS tests (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id TEXT NOT NULL,
            path        TEXT NOT NULL,
            name        TEXT NOT NULL,
            capability  TEXT
        );
        CREATE TABLE IF NOT EXISTS docs (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id TEXT NOT NULL,
            path        TEXT NOT NULL,
            content     TEXT NOT NULL DEFAULT '',
            capability  TEXT
        );
        CREATE TABLE IF NOT EXISTS capabilities (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id  TEXT NOT NULL,
            capability   TEXT NOT NULL,
            file_path    TEXT NOT NULL,
            line         INTEGER NOT NULL DEFAULT 0,
            matched_term TEXT NOT NULL DEFAULT '',
            classification TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS patterns (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id  TEXT NOT NULL,
            pattern      TEXT NOT NULL,
            locations    INTEGER NOT NULL DEFAULT 0,
            ev_types     TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS classifications (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_run_id  TEXT NOT NULL,
            file_path    TEXT NOT NULL,
            capability   TEXT NOT NULL,
            classification TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS receipts (
            id               TEXT PRIMARY KEY,
            scan_run_id      TEXT NOT NULL,
            timestamp        TEXT NOT NULL,
            roots            TEXT NOT NULL,
            file_count       INTEGER NOT NULL DEFAULT 0,
            total_bytes      INTEGER NOT NULL DEFAULT 0,
            root_hash        TEXT NOT NULL DEFAULT '',
            schema_version   TEXT NOT NULL DEFAULT '1.0.0',
            hostname         TEXT,
            os               TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_cap_name ON capabilities(capability);
        CREATE INDEX IF NOT EXISTS idx_cap_scan ON capabilities(scan_run_id);
        CREATE INDEX IF NOT EXISTS idx_files_scan ON files(scan_run_id);
    "#).context("schema migration")?;
    Ok(())
}

pub fn begin_scan(conn: &Connection, id: &str, roots: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO scan_runs (id, started_at, roots, status) VALUES (?1, ?2, ?3, 'running')",
        params![id, now, roots],
    ).context("begin_scan")?;
    Ok(())
}

pub fn finish_scan(conn: &Connection, id: &str, file_count: usize, total_bytes: u64) -> Result<()> {
    conn.execute(
        "UPDATE scan_runs SET file_count=?2, total_bytes=?3, status='done' WHERE id=?1",
        params![id, file_count as i64, total_bytes as i64],
    ).context("finish_scan")?;
    Ok(())
}

pub fn insert_repo(conn: &Connection, scan_id: &str, root: &str, name: &str, ecosystem: &str, git_head: Option<&str>) -> Result<()> {
    conn.execute(
        "INSERT INTO repositories (scan_run_id, root, name, ecosystem, git_head) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![scan_id, root, name, ecosystem, git_head],
    ).context("insert_repo")?;
    Ok(())
}

pub fn insert_file(
    conn: &Connection, scan_id: &str,
    f: &crate::models::FileEntry,
    is_test: bool, is_doc: bool, is_config: bool, is_script: bool, is_binary: bool,
    git_root: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO files (scan_run_id, path, size, hash, language, is_test, is_doc, is_config, is_script, is_binary, git_root)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            scan_id, f.path.display().to_string(), f.size as i64, f.hash, f.language,
            is_test as i32, is_doc as i32, is_config as i32, is_script as i32, is_binary as i32,
            git_root,
        ],
    ).context("insert_file")?;
    Ok(())
}

pub fn insert_symbol(conn: &Connection, scan_id: &str, sym: &Symbol) -> Result<()> {
    conn.execute(
        "INSERT INTO symbols (scan_run_id, name, kind, line, file_path) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![scan_id, sym.name, sym.kind, sym.line as i64, sym.file_path.display().to_string()],
    ).context("insert_symbol")?;
    Ok(())
}

pub fn insert_dependency(conn: &Connection, scan_id: &str, name: &str, version: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO dependencies (scan_run_id, name, version) VALUES (?1, ?2, ?3)",
        params![scan_id, name, version],
    ).context("insert_dependency")?;
    Ok(())
}

pub fn insert_test(conn: &Connection, scan_id: &str, path: &str, name: &str, capability: Option<&str>) -> Result<()> {
    conn.execute(
        "INSERT INTO tests (scan_run_id, path, name, capability) VALUES (?1, ?2, ?3, ?4)",
        params![scan_id, path, name, capability],
    ).context("insert_test")?;
    Ok(())
}

pub fn insert_doc(conn: &Connection, scan_id: &str, path: &str, content: &str, capability: Option<&str>) -> Result<()> {
    // Store only first 2000 chars of content to keep DB size reasonable
    let excerpt = content.chars().take(2000).collect::<String>();
    conn.execute(
        "INSERT INTO docs (scan_run_id, path, content, capability) VALUES (?1, ?2, ?3, ?4)",
        params![scan_id, path, excerpt, capability],
    ).context("insert_doc")?;
    Ok(())
}

pub fn insert_capability(conn: &Connection, scan_id: &str, cap: &DetectedCapability) -> Result<()> {
    conn.execute(
        "INSERT INTO capabilities (scan_run_id, capability, file_path, line, matched_term, classification)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            scan_id, cap.capability,
            cap.file_path.display().to_string(),
            cap.line as i64, cap.matched_term,
            cap.classification.to_string(),
        ],
    ).context("insert_capability")?;
    Ok(())
}

/// Also expose insert_capability_hit as an alias for backward compat
pub fn insert_capability_hit(conn: &Connection, scan_id: &str, cap: &DetectedCapability) -> Result<()> {
    insert_capability(conn, scan_id, cap)
}

pub fn insert_pattern(conn: &Connection, scan_id: &str, pattern: &str, locations: i64, ev_types: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO patterns (scan_run_id, pattern, locations, ev_types) VALUES (?1, ?2, ?3, ?4)",
        params![scan_id, pattern, locations, ev_types],
    ).context("insert_pattern")?;
    Ok(())
}

pub fn insert_classification(conn: &Connection, scan_id: &str, file_path: &str, capability: &str, classification: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO classifications (scan_run_id, file_path, capability, classification) VALUES (?1, ?2, ?3, ?4)",
        params![scan_id, file_path, capability, classification],
    ).context("insert_classification")?;
    Ok(())
}

pub fn insert_receipt(conn: &Connection, scan_id: &str, r: &ScanReceipt) -> Result<()> {
    let roots = r.root_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ");
    conn.execute(
        "INSERT OR REPLACE INTO receipts (id, scan_run_id, timestamp, roots, file_count, total_bytes, root_hash, schema_version, hostname, os)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            r.id, scan_id, r.timestamp, roots,
            r.file_count as i64, r.total_bytes as i64,
            r.root_hash, r.schema_version,
            r.system_info.hostname, r.system_info.os,
        ],
    ).context("insert_receipt")?;
    Ok(())
}

pub fn query_capability(conn: &Connection, name: &str) -> Result<Vec<(String, String, Option<i64>, String, String, f64)>> {
    let pattern = format!("%{}%", name.to_lowercase());
    let mut stmt = conn.prepare(
        "SELECT capability, file_path, line, classification, matched_term, 0.8 as confidence
         FROM capabilities WHERE LOWER(capability) LIKE ?1
         ORDER BY capability LIMIT 100"
    ).context("query_capability")?;
    let rows = stmt.query_map(params![pattern], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get::<_, Option<i64>>(2)?, r.get(3)?, r.get(4)?, r.get::<_, f64>(5)?))
    }).context("query_capability rows")?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

pub fn get_summary(conn: &Connection) -> Result<(i64, i64, i64, i64)> {
    let runs: i64 = conn.query_row("SELECT COUNT(*) FROM scan_runs", [], |r| r.get(0)).unwrap_or(0);
    let caps: i64 = conn.query_row("SELECT COUNT(DISTINCT capability) FROM capabilities", [], |r| r.get(0)).unwrap_or(0);
    let receipts: i64 = conn.query_row("SELECT COUNT(*) FROM receipts", [], |r| r.get(0)).unwrap_or(0);
    let total_hits: i64 = conn.query_row("SELECT COUNT(*) FROM capabilities", [], |r| r.get(0)).unwrap_or(0);
    Ok((runs, caps, receipts, total_hits))
}

pub fn integrity_check(conn: &Connection) -> Result<bool> {
    let result: String = conn.query_row("PRAGMA integrity_check;", [], |r| r.get(0))
        .context("integrity_check")?;
    Ok(result == "ok")
}

pub fn list_receipts(conn: &Connection) -> Result<Vec<(String, String, i64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, file_count, roots FROM receipts ORDER BY timestamp DESC LIMIT 50"
    ).context("list_receipts")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .context("list_receipts rows")?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}
