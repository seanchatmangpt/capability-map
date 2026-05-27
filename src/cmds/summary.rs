use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct SummaryArgs {
    /// Catalog database path
    #[arg(long, default_value = ".cpmp/workspace.sqlite")]
    pub db: PathBuf,
}

pub fn run(args: SummaryArgs) -> anyhow::Result<()> {
    if !args.db.exists() {
        return Err(anyhow::anyhow!(
            "No catalog found at: {}\n\nNext:\n  cpmp computer discover <path> --out .cpmp",
            args.db.display()
        ));
    }
    let conn = crate::db::open(&args.db)?;
    let (runs, caps, receipts, total_hits) = crate::db::get_summary(&conn)?;
    let ok = crate::db::integrity_check(&conn).unwrap_or(false);

    println!("cpmp catalog summary");
    println!("  db:            {}", args.db.display());
    println!("  scan runs:     {}", runs);
    println!("  capabilities:  {} distinct ({} hits)", caps, total_hits);
    println!("  receipts:      {}", receipts);
    println!("  integrity:     {}", if ok { "ok ✓" } else { "FAILED ✗" });
    println!();
    println!("Next:");
    println!("  cpmp capability list --db {}", args.db.display());
    println!("  cpmp capability find <name> --db {}", args.db.display());
    println!("  cpmp receipt list --db {}", args.db.display());
    Ok(())
}
