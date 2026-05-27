use clap::{Args, Subcommand};
use std::path::PathBuf;
use anyhow::{bail, Result};

#[derive(Args)]
pub struct ComputerArgs {
    #[command(subcommand)]
    pub command: ComputerCmd,
}

#[derive(Subcommand)]
pub enum ComputerCmd {
    /// Discover all projects and capabilities under given paths
    Discover {
        /// Paths to scan (space-separated)
        paths: Vec<PathBuf>,
        /// Output directory for catalog (default: .cpmp)
        #[arg(long, default_value = ".cpmp")]
        out: PathBuf,
        /// Skip SQLite persistence
        #[arg(long)]
        no_db: bool,
    },
}

pub fn run(args: ComputerArgs) -> Result<()> {
    match args.command {
        ComputerCmd::Discover { paths, out, no_db } => discover(paths, out, no_db),
    }
}

fn discover(paths: Vec<PathBuf>, out: PathBuf, no_db: bool) -> Result<()> {
    if paths.is_empty() {
        bail!("No paths given.\n\nUsage:\n  cpmp computer discover <path...> --out .cpmp");
    }

    // Refusal: validate all paths exist before touching anything
    for p in &paths {
        if !p.exists() {
            bail!("REFUSAL: Path not found: {}\n\nCannot scan a path that does not exist.", p.display());
        }
    }

    println!("cpmp computer discover");
    println!("  roots: {}", paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "));
    println!("  out:   {}", out.display());
    println!();

    let start = std::time::Instant::now();

    // Delegate to the swarm's scanner — it handles walking, hashing, capability detection,
    // receipt generation, RDF catalog emission, and JSON reports.
    let receipt = crate::scanner::scan(&paths, &out)?;

    let elapsed = start.elapsed();
    println!("  files:   {}", receipt.file_count);
    println!("  bytes:   {}", receipt.total_bytes);
    println!("  time:    {:.2}s", elapsed.as_secs_f64());

    // Persist to SQLite (optional — sqlite is the acceleration cache, not the source store)
    if !no_db {
        let db_path = out.join("workspace.sqlite");
        let conn = crate::db::open(&db_path)?;
        let scan_id = receipt.id.clone();
        let roots_str = paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ");
        crate::db::begin_scan(&conn, &scan_id, &roots_str)?;

        // Insert the capability hits into SQLite from JSON reports
        let inv_path = out.join("reports").join("capability_inventory.json");
        if inv_path.exists() {
            let json = std::fs::read_to_string(&inv_path)?;
            let caps: Vec<crate::models::DetectedCapability> = serde_json::from_str(&json)
                .unwrap_or_default();
            for cap in &caps {
                crate::db::insert_capability_hit(&conn, &scan_id, cap)?;
            }
            println!("  capabilities: {}", caps.len());
        }

        crate::db::finish_scan(&conn, &scan_id, receipt.file_count, receipt.total_bytes)?;
        crate::db::insert_receipt(&conn, &scan_id, &receipt)?;

        // Emit Markdown reports from SQLite
        let reports_dir = out.join("reports");
        let report_paths = crate::report::emit_all(&conn, &reports_dir)?;
        println!("Reports:");
        for p in &report_paths {
            println!("  {}", p.display());
        }

        println!("Catalog: {}", db_path.display());
    }

    let receipt_path = out.join("receipts");
    println!("Receipts: {}", receipt_path.display());
    println!("RDF:      {}", out.join("catalog").display());
    println!("\nNext:");
    println!("  cpmp summary --db {}", out.join("workspace.sqlite").display());
    println!("  cpmp capability find <name> --db {}", out.join("workspace.sqlite").display());

    Ok(())
}
