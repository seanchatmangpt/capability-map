use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct ReportArgs {
    #[command(subcommand)]
    pub command: ReportCmd,
}

#[derive(Subcommand)]
pub enum ReportCmd {
    /// Emit all Markdown reports from catalog
    Emit {
        #[arg(long, default_value = ".cpmp/workspace.sqlite")]
        db: PathBuf,
        #[arg(long, default_value = ".cpmp/reports")]
        out: PathBuf,
    },
}

pub fn run(args: ReportArgs) -> anyhow::Result<()> {
    match args.command {
        ReportCmd::Emit { db, out } => emit(db, out),
    }
}

fn emit(db: PathBuf, out: PathBuf) -> anyhow::Result<()> {
    if !db.exists() {
        return Err(anyhow::anyhow!(
            "No catalog found at: {}\n\nNext:\n  cpmp computer discover <path> --out .cpmp",
            db.display()
        ));
    }
    let conn = crate::db::open(&db).map_err(|e| anyhow::anyhow!("{}", e))?;
    let paths = crate::report::emit_all(&conn, &out).map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("Reports emitted:");
    for p in &paths {
        println!("  {}", p.display());
    }
    Ok(())
}
