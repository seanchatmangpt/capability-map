use std::path::Path;
use crate::error::Result;

/// Emit all Markdown reports from the catalog database.
pub fn emit_all(conn: &rusqlite::Connection, reports_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(reports_dir)?;
    let mut paths = vec![];
    paths.push(emit_capability_inventory(conn, reports_dir)?);
    paths.push(emit_pattern_atlas(conn, reports_dir)?);
    paths.push(emit_test_evidence(conn, reports_dir)?);
    paths.push(emit_doc_claims(conn, reports_dir)?);
    Ok(paths)
}

fn emit_capability_inventory(conn: &rusqlite::Connection, dir: &Path) -> Result<std::path::PathBuf> {
    let mut md = String::from("# CAPABILITY_INVENTORY\n\n");
    md.push_str("| Capability | Files | Tests | Docs | Top Evidence | Confidence |\n");
    md.push_str("| ---------- | ----: | ----: | ---: | ------------ | ---------: |\n");

    let mut stmt = conn.prepare(
        "SELECT capability,
                COUNT(DISTINCT file_path) as files,
                SUM(CASE WHEN evidence_type='TEST' THEN 1 ELSE 0 END) as tests,
                SUM(CASE WHEN evidence_type='DOC' THEN 1 ELSE 0 END) as docs,
                MAX(evidence_type) as top_evidence,
                MAX(confidence) as max_confidence
         FROM capabilities
         GROUP BY capability
         ORDER BY files DESC, max_confidence DESC"
    )?;

    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, i64>(2)?,
            r.get::<_, i64>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, f64>(5)?,
        ))
    })?;

    for row in rows.flatten() {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.0}% |\n",
            row.0, row.1, row.2, row.3, row.4, row.5 * 100.0
        ));
    }

    let path = dir.join("CAPABILITY_INVENTORY.md");
    std::fs::write(&path, md)?;
    Ok(path)
}

fn emit_pattern_atlas(conn: &rusqlite::Connection, dir: &Path) -> Result<std::path::PathBuf> {
    let mut md = String::from("# PATTERN_ATLAS\n\n");
    md.push_str("| Pattern | Locations | Evidence Types | Finish Action |\n");
    md.push_str("| ------- | --------: | -------------- | ------------- |\n");

    let mut stmt = conn.prepare(
        "SELECT capability,
                COUNT(DISTINCT file_path) as locs,
                GROUP_CONCAT(DISTINCT evidence_type) as evs
         FROM capabilities
         GROUP BY capability
         ORDER BY locs DESC"
    )?;

    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, String>(2)?))
    })?;

    for row in rows.flatten() {
        let finish = if row.1 == 1 { "Widen coverage" } else { "Verify tests pass" };
        md.push_str(&format!("| {} | {} | {} | {} |\n", row.0, row.1, row.2, finish));
    }

    let path = dir.join("PATTERN_ATLAS.md");
    std::fs::write(&path, md)?;
    Ok(path)
}

fn emit_test_evidence(conn: &rusqlite::Connection, dir: &Path) -> Result<std::path::PathBuf> {
    let mut md = String::from("# TEST_EVIDENCE_MAP\n\n");
    md.push_str("| Capability | Test File | Line | Context |\n");
    md.push_str("| ---------- | --------- | ---: | ------- |\n");

    let mut stmt = conn.prepare(
        "SELECT capability, file_path, line_number, context FROM capabilities WHERE evidence_type='TEST' ORDER BY capability, file_path LIMIT 500"
    )?;

    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<i64>>(2)?,
            r.get::<_, String>(3)?,
        ))
    })?;

    for row in rows.flatten() {
        let line = row.2.map(|n| n.to_string()).unwrap_or_else(|| "—".into());
        let ctx = row.3.chars().take(60).collect::<String>();
        md.push_str(&format!("| {} | {} | {} | {} |\n", row.0, row.1, line, ctx));
    }

    let path = dir.join("TEST_EVIDENCE_MAP.md");
    std::fs::write(&path, md)?;
    Ok(path)
}

fn emit_doc_claims(conn: &rusqlite::Connection, dir: &Path) -> Result<std::path::PathBuf> {
    let mut md = String::from("# DOC_CLAIM_MAP\n\n");
    md.push_str("| Capability | Doc File | Line | Context |\n");
    md.push_str("| ---------- | -------- | ---: | ------- |\n");

    let mut stmt = conn.prepare(
        "SELECT capability, file_path, line_number, context FROM capabilities WHERE evidence_type='DOC' ORDER BY capability, file_path LIMIT 500"
    )?;

    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<i64>>(2)?,
            r.get::<_, String>(3)?,
        ))
    })?;

    for row in rows.flatten() {
        let line = row.2.map(|n| n.to_string()).unwrap_or_else(|| "—".into());
        let ctx = row.3.chars().take(60).collect::<String>();
        md.push_str(&format!("| {} | {} | {} | {} |\n", row.0, row.1, line, ctx));
    }

    let path = dir.join("DOC_CLAIM_MAP.md");
    std::fs::write(&path, md)?;
    Ok(path)
}
