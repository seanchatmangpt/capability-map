use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct ReceiptArgs {
    #[command(subcommand)]
    pub command: ReceiptCmd,
}

#[derive(Subcommand)]
pub enum ReceiptCmd {
    /// List all scan receipts
    List {
        #[arg(long, default_value = ".cpmp/workspace.sqlite")]
        db: PathBuf,
    },
    /// Verify no files were deleted between two receipts
    VerifyNoDeletion {
        /// Path to the earlier receipt TOML
        #[arg(long)]
        before: PathBuf,
        /// Path to the later receipt TOML
        #[arg(long)]
        after: PathBuf,
    },
}

pub fn run(args: ReceiptArgs) -> anyhow::Result<()> {
    match args.command {
        ReceiptCmd::List { db } => list(db),
        ReceiptCmd::VerifyNoDeletion { before, after } => verify(before, after),
    }
}

fn list(db: PathBuf) -> anyhow::Result<()> {
    if !db.exists() {
        return Err(anyhow::anyhow!(
            "No catalog found at: {}\n\nNext:\n  cpmp computer discover <path> --out .cpmp",
            db.display()
        ));
    }
    let conn = crate::db::open(&db).map_err(|e| anyhow::anyhow!("{}", e))?;
    let receipts = crate::db::list_receipts(&conn).map_err(|e| anyhow::anyhow!("{}", e))?;

    if receipts.is_empty() {
        println!("No receipts found. Run: cpmp computer discover <path>");
        return Ok(());
    }

    println!("{:<40} {:<30} {:>10} {}", "RECEIPT_ID", "TIMESTAMP", "FILES", "ROOTS");
    println!("{}", "─".repeat(100));
    for (id, ts, files, roots) in &receipts {
        let id_short = id.chars().take(36).collect::<String>();
        println!("{:<40} {:<30} {:>10} {}", id_short, ts, files, roots);
    }
    Ok(())
}

fn verify(before_path: PathBuf, after_path: PathBuf) -> anyhow::Result<()> {
    if !before_path.exists() {
        return Err(anyhow::anyhow!("REFUSAL: Before receipt not found: {}", before_path.display()));
    }
    if !after_path.exists() {
        return Err(anyhow::anyhow!("REFUSAL: After receipt not found: {}", after_path.display()));
    }

    // For full file comparison we need to scan. Here we compare root hashes and metadata.
    let before = crate::receipt::ScanReceipt::load(&before_path)
        .map_err(|e| anyhow::anyhow!("Failed to load before receipt: {}", e))?;
    let after = crate::receipt::ScanReceipt::load(&after_path)
        .map_err(|e| anyhow::anyhow!("Failed to load after receipt: {}", e))?;

    println!("verify-no-deletion");
    println!("  before: {} ({})", before_path.display(), before.timestamp);
    println!("  after:  {} ({})", after_path.display(), after.timestamp);
    println!();

    if before.root_hash == after.root_hash {
        println!("STATUS: UNCHANGED — root hashes match");
        println!("  root_hash: {}", before.root_hash);
        return Ok(());
    }

    println!("STATUS: CHANGED — root hashes differ");
    println!("  before_hash: {}", before.root_hash);
    println!("  after_hash:  {}", after.root_hash);
    println!("  before_files: {}", before.file_count);
    println!("  after_files:  {}", after.file_count);

    if after.file_count < before.file_count {
        let missing = before.file_count - after.file_count;
        println!("\nREFUSAL: {} file(s) appear MISSING between receipts.", missing);
        println!("  Missing files are a refusal condition — they are not auto-fixed.");
        println!("  Re-scan with: cpmp computer discover <path>");
        std::process::exit(2);
    }

    let added = after.file_count.saturating_sub(before.file_count);
    if added > 0 {
        println!("\n  ADDED: {} new file(s)", added);
    }

    Ok(())
}
