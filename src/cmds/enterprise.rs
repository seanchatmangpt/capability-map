use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct EnterpriseArgs {
    #[command(subcommand)]
    pub command: EnterpriseCmd,
}

#[derive(Subcommand)]
pub enum EnterpriseCmd {
    /// Run doctor checks: validate catalog, graph, integrity, gates
    Doctor {
        #[arg(long, default_value = ".cpmp")]
        out: PathBuf,
    },
}

pub fn run(args: EnterpriseArgs) -> anyhow::Result<()> {
    match args.command {
        EnterpriseCmd::Doctor { out } => doctor(out),
    }
}

fn doctor(out: PathBuf) -> anyhow::Result<()> {
    println!("cpmp enterprise doctor");
    println!("  catalog: {}", out.display());
    println!();

    let db = out.join("workspace.sqlite");
    let catalog_ttl = out.join("catalog/cpmp-catalog.ttl");
    let receipts = out.join("receipts");
    let reports = out.join("reports");

    let checks: &[(&str, bool, &str)] = &[
        ("catalog directory exists",  out.exists(),          "run: cpmp computer discover <path> --out .cpmp"),
        ("workspace.sqlite exists",   db.exists(),           "run: cpmp computer discover <path> --out .cpmp"),
        ("receipts directory exists", receipts.exists(),     "run: cpmp computer discover <path> --out .cpmp"),
        ("reports directory exists",  reports.exists(),      "run: cpmp computer discover <path> --out .cpmp"),
        ("catalog TTL projected",     catalog_ttl.exists(),  "run: cpmp graph project --db .cpmp/workspace.sqlite"),
    ];

    let db_integrity = if db.exists() {
        crate::db::open(&db)
            .and_then(|conn| crate::db::integrity_check(&conn))
            .unwrap_or(false)
    } else {
        false
    };

    let capability_ok = if db.exists() {
        crate::db::open(&db)
            .and_then(|conn| crate::db::get_summary(&conn))
            .map(|(_, caps, _, _)| caps > 0)
            .unwrap_or(false)
    } else {
        false
    };

    let receipt_ok = if receipts.exists() {
        std::fs::read_dir(&receipts).map(|mut d| d.next().is_some()).unwrap_or(false)
    } else {
        false
    };

    let mut pass = 0usize;
    let mut fail = 0usize;

    for (name, ok, hint) in checks {
        if *ok { println!("  ✓ {}", name); pass += 1; }
        else   { println!("  ✗ {} — {}", name, hint); fail += 1; }
    }

    if db.exists() {
        if db_integrity  { println!("  ✓ SQLite integrity check"); pass += 1; }
        else             { println!("  ✗ SQLite integrity check FAILED"); fail += 1; }
        if capability_ok { println!("  ✓ capabilities detected"); pass += 1; }
        else             { println!("  ✗ no capabilities detected — run discover"); fail += 1; }
    }

    if receipt_ok { println!("  ✓ receipts present"); pass += 1; }
    else          { println!("  ✗ no receipts — run discover"); fail += 1; }

    println!();
    println!("PASS: {}  FAIL: {}", pass, fail);
    println!();

    if fail == 0 {
        println!("STATUS: A — catalog ready.");
    } else if pass > fail {
        println!("STATUS: B — partially complete; {} gap(s) remain.", fail);
    } else {
        println!("STATUS: C — core setup required.");
    }

    Ok(())
}
