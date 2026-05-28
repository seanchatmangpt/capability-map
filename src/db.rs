use std::path::PathBuf;
use anyhow::Result;
use rusqlite::Connection;
use crate::models::{FileEntry, Symbol, DetectedCapability};
pub fn init_db(out: &PathBuf) -> Result<()> {
    let db_path = out.join("workspace.sqlite");
    let conn = Connection::open(&db_path)?;
    conn.execute("CREATE TABLE IF NOT EXISTS files (path TEXT PRIMARY KEY, hash TEXT NOT NULL, size INTEGER NOT NULL, language TEXT NOT NULL)", [])?;
    conn.execute("CREATE TABLE IF NOT EXISTS symbols (file_path TEXT, name TEXT, kind TEXT)", [])?;
    conn.execute("CREATE TABLE IF NOT EXISTS capabilities (file_path TEXT, capability TEXT, matched_term TEXT, classification TEXT)", [])?;
    Ok(())
}
pub fn insert_files(out: &PathBuf, files: &[FileEntry]) -> Result<()> {
    let db_path = out.join("workspace.sqlite");
    let mut conn = Connection::open(&db_path)?;
    let tx = conn.transaction()?;
    for f in files { tx.execute("INSERT OR REPLACE INTO files (path, hash, size, language) VALUES (?1, ?2, ?3, ?4)", (&f.path, &f.hash, &f.size, &f.language))?; }
    tx.commit()?; Ok(())
}
pub fn insert_symbols(out: &PathBuf, symbols: &[Symbol]) -> Result<()> {
    let db_path = out.join("workspace.sqlite");
    let mut conn = Connection::open(&db_path)?;
    let tx = conn.transaction()?;
    for s in symbols { tx.execute("INSERT INTO symbols (file_path, name, kind) VALUES (?1, ?2, ?3)", (&s.file_path, &s.name, &s.kind))?; }
    tx.commit()?; Ok(())
}
pub fn insert_capabilities(out: &PathBuf, caps: &[DetectedCapability]) -> Result<()> {
    let db_path = out.join("workspace.sqlite");
    let mut conn = Connection::open(&db_path)?;
    let tx = conn.transaction()?;
    for c in caps { tx.execute("INSERT INTO capabilities (file_path, capability, matched_term, classification) VALUES (?1, ?2, ?3, ?4)", (&c.file_path, &c.capability, &c.matched_term, &c.classification))?; }
    tx.commit()?; Ok(())
}
