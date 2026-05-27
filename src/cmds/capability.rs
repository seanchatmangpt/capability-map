use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct CapabilityArgs {
    #[command(subcommand)]
    pub command: CapabilityCmd,
}

#[derive(Subcommand)]
pub enum CapabilityCmd {
    /// Find all occurrences of a named capability
    Find {
        /// Capability name to search (case-insensitive)
        name: String,
        /// Catalog database path
        #[arg(long, default_value = ".cpmp/workspace.sqlite")]
        db: PathBuf,
        /// Limit results
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// List all detected capabilities
    List {
        #[arg(long, default_value = ".cpmp/workspace.sqlite")]
        db: PathBuf,
    },
}

pub fn run(args: CapabilityArgs) -> anyhow::Result<()> {
    match args.command {
        CapabilityCmd::Find { name, db, limit } => find(name, db, limit),
        CapabilityCmd::List { db } => list(db),
    }
}

fn require_db(db: &PathBuf) -> anyhow::Result<rusqlite::Connection> {
    if !db.exists() {
        return Err(anyhow::anyhow!(
            "No catalog found at: {}\n\nNext:\n  cpmp computer discover <path> --out .cpmp",
            db.display()
        ));
    }
    crate::db::open(db).map_err(|e| anyhow::anyhow!("{}", e))
}

fn find(name: String, db: PathBuf, limit: usize) -> anyhow::Result<()> {
    let conn = require_db(&db)?;
    let hits = crate::db::query_capability(&conn, &name)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if hits.is_empty() {
        println!("REFUSAL: No evidence found for capability '{}'", name);
        println!("\nNext:");
        println!("  cpmp capability list --db {}", db.display());
        return Ok(());
    }

    println!("# Capability: {}\n", name);
    println!("{:<15} {:<60} {:>6} {:<10} {:>8}", "CAPABILITY", "FILE", "LINE", "EVIDENCE", "CONF%");
    println!("{}", "─".repeat(105));

    for (cap, file, line, evtype, ctx, conf) in hits.iter().take(limit) {
        let line_str = line.map(|n| n.to_string()).unwrap_or_else(|| "—".into());
        let file_short = file.chars().rev().take(55).collect::<String>().chars().rev().collect::<String>();
        println!("{:<15} {:<60} {:>6} {:<10} {:>7.0}%",
            cap, file_short, line_str, evtype, conf * 100.0);
        if !ctx.is_empty() {
            println!("  context: {}", ctx.chars().take(80).collect::<String>());
        }
    }

    println!("\n{} results", hits.len().min(limit));
    Ok(())
}

fn list(db: PathBuf) -> anyhow::Result<()> {
    let conn = require_db(&db)?;
    let mut stmt = conn.prepare(
        "SELECT capability, COUNT(DISTINCT file_path) as files FROM capabilities GROUP BY capability ORDER BY files DESC"
    )?;
    let rows: Vec<(String, i64)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?.filter_map(|r| r.ok()).collect();

    if rows.is_empty() {
        println!("No capabilities detected yet. Run: cpmp computer discover <path>");
        return Ok(());
    }

    println!("{:<20} {:>8}", "CAPABILITY", "FILES");
    println!("{}", "─".repeat(30));
    for (cap, files) in &rows {
        println!("{:<20} {:>8}", cap, files);
    }
    println!("\n{} capabilities total", rows.len());
    Ok(())
}
