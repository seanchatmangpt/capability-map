use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use cpmp::{gates, policy, projection, receipt, scanner};
use std::path::PathBuf;

// ─── CLI definition ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name  = "cpmp",
    about = "Computer Project Mapping Protocol — enterprise capability surveyor",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Project discovery and computer inventory
    Computer {
        #[command(subcommand)]
        sub: ComputerCmd,
    },
    /// Graph operations (validate / load / query / version / drift)
    Graph {
        #[command(subcommand)]
        sub: GraphCmd,
    },
    /// Policy enforcement
    Policy {
        #[command(subcommand)]
        sub: PolicyCmd,
    },
    /// Tenant management
    Tenant {
        #[command(subcommand)]
        sub: TenantCmd,
    },
    /// Audit and lineage
    Audit {
        #[command(subcommand)]
        sub: AuditCmd,
    },
    /// Receipt lifecycle
    Receipt {
        #[command(subcommand)]
        sub: ReceiptCmd,
    },
    /// Enterprise diagnostics
    Enterprise {
        #[command(subcommand)]
        sub: EnterpriseCmd,
    },
}

// ── Computer ─────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ComputerCmd {
    /// Discover and scan project directories
    Discover {
        /// Paths to scan
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Output directory (default: ~/.cpmp/catalog)
        #[arg(long, short)]
        out: Option<PathBuf>,
        /// Run Open Ontologies admission gates after scan
        #[arg(long)]
        with_gates: bool,
    },
}

// ── Graph ─────────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum GraphCmd {
    /// Validate Turtle file via open-ontologies
    Validate {
        #[arg(required = true)]
        file: PathBuf,
    },
    /// Load Turtle file into Open Ontologies store
    Load {
        #[arg(required = true)]
        file: PathBuf,
    },
    /// Run SPARQL query against open-ontologies
    Query {
        /// SPARQL query string or @file
        #[arg(required = true)]
        sparql: String,
    },
    /// Version the current graph state
    Version {
        #[arg(required = true)]
        label: String,
    },
    /// Detect drift between two Turtle files
    Drift {
        before: PathBuf,
        after: PathBuf,
    },
    /// Project discovered data to RDF (without scanning filesystem)
    Project {
        /// JSON file inventory file
        #[arg(long, required = true)]
        files: PathBuf,
        /// Output directory
        #[arg(long, required = true)]
        out: PathBuf,
    },
}

// ── Policy ───────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum PolicyCmd {
    /// Run all policy checks against a catalog directory
    Check {
        #[arg(long, required = true)]
        catalog: PathBuf,
    },
    /// Enforce policy: exit 1 if any pack fails
    Enforce {
        #[arg(long, required = true)]
        catalog: PathBuf,
    },
}

// ── Tenant ───────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum TenantCmd {
    /// Create a new tenant namespace
    Create {
        #[arg(required = true)]
        name: String,
    },
    /// List all tenant namespaces
    List,
}

// ── Audit ────────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum AuditCmd {
    /// Show lineage trail from open-ontologies
    Lineage {
        #[arg(long)]
        limit: Option<usize>,
    },
}

// ── Receipt ──────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ReceiptCmd {
    /// Emit a receipt for a directory
    Emit {
        #[arg(required = true)]
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Verify no files were deleted between two receipt files
    VerifyNoDeletion {
        before: PathBuf,
        after: PathBuf,
    },
}

// ── Enterprise ───────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum EnterpriseCmd {
    /// Run enterprise diagnostics and report readiness
    Doctor {
        #[arg(long)]
        catalog: Option<PathBuf>,
    },
}

// ─── main ────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Computer { sub } => match sub {
            ComputerCmd::Discover { paths, out, with_gates } => {
                let out_dir = out.unwrap_or_else(default_cpmp_dir);
                let rec = scanner::scan(&paths, &out_dir)?;
                let catalog_dir = out_dir.join("catalog");

                if with_gates {
                    let hash = gates::run_admission_gates(&catalog_dir, &rec)?;
                    let reports_dir = out_dir.join("reports");
                    projection::emit_all(&reports_dir, &hash)?;
                    println!("Catalog hash: {hash}");
                } else {
                    eprintln!("Tip: use --with-gates to run Open Ontologies admission gates.");
                }

                println!("Scan receipt: {}", rec.id);
                println!("Output dir: {}", out_dir.display());
            }
        },

        Commands::Graph { sub } => match sub {
            GraphCmd::Validate { file } => {
                gates::gate_validate(&file)?;
                println!("Validation passed: {:?}", file);
            }
            GraphCmd::Load { file } => {
                gates::gate_load(&file)?;
                println!("Loaded: {:?}", file);
            }
            GraphCmd::Query { sparql } => {
                let status = std::process::Command::new("open-ontologies")
                    .arg("query")
                    .arg(&sparql)
                    .status()?;
                if !status.success() {
                    bail!("SPARQL query failed");
                }
            }
            GraphCmd::Version { label } => {
                gates::gate_version(&label)?;
                println!("Versioned: {label}");
            }
            GraphCmd::Drift { before, after } => {
                let status = std::process::Command::new("open-ontologies")
                    .arg("diff")
                    .arg(&before)
                    .arg(&after)
                    .status()?;
                if !status.success() {
                    bail!("Drift detection failed");
                }
            }
            GraphCmd::Project { files, out } => {
                let raw = std::fs::read_to_string(&files)?;
                let file_entries: Vec<cpmp::models::FileEntry> = serde_json::from_str(&raw)?;
                let empty_caps = vec![];
                let root = out.clone();
                let rec = receipt::generate_receipt(&root, &file_entries)?;
                cpmp::rdf::build_and_emit(&file_entries, &empty_caps, &rec, &out)?;
                println!("RDF projected to {:?}", out);
            }
        },

        Commands::Policy { sub } => match sub {
            PolicyCmd::Check { catalog } => {
                let checks = policy::run_policy_checks(&catalog);
                policy::print_policy_report(&checks);
            }
            PolicyCmd::Enforce { catalog } => {
                let checks = policy::run_policy_checks(&catalog);
                let code = policy::print_policy_report(&checks);
                if code != 0 {
                    bail!("Policy enforcement failed");
                }
                println!("All policy packs passed.");
            }
        },

        Commands::Tenant { sub } => match sub {
            TenantCmd::Create { name } => {
                // Stub: tenant metadata stored as JSON in ~/.cpmp/tenants/
                let tenants_dir = dirs_home().join(".cpmp").join("tenants");
                std::fs::create_dir_all(&tenants_dir)?;
                let tenant_file = tenants_dir.join(format!("{name}.json"));
                let meta = serde_json::json!({
                    "name": name,
                    "created": chrono::Utc::now().to_rfc3339(),
                    "namespace": format!("urn:cpmp:tenant:{name}"),
                });
                std::fs::write(&tenant_file, serde_json::to_string_pretty(&meta)?)?;
                println!("Tenant created: {name}");
            }
            TenantCmd::List => {
                let tenants_dir = dirs_home().join(".cpmp").join("tenants");
                if !tenants_dir.exists() {
                    println!("No tenants found.");
                    return Ok(());
                }
                for entry in std::fs::read_dir(&tenants_dir)? {
                    let e = entry?;
                    println!("{}", e.file_name().to_string_lossy());
                }
            }
        },

        Commands::Audit { sub } => match sub {
            AuditCmd::Lineage { limit } => {
                let mut cmd = std::process::Command::new("open-ontologies");
                cmd.arg("lineage");
                if let Some(n) = limit {
                    cmd.arg("--limit").arg(n.to_string());
                }
                let status = cmd.status()?;
                if !status.success() {
                    bail!("Lineage command failed");
                }
            }
        },

        Commands::Receipt { sub } => match sub {
            ReceiptCmd::Emit { path, out } => {
                let out_dir = out.unwrap_or_else(|| path.clone());
                let entries = scan_to_entries(&path)?;
                let rec = receipt::generate_receipt(&path, &entries)?;
                let rec_path = out_dir.join(format!("scan-{}.receipt.toml", rec.id));
                std::fs::create_dir_all(&out_dir)?;
                std::fs::write(&rec_path, toml::to_string(&rec)?)?;
                println!("Receipt emitted: {:?}", rec_path);
                println!("ID: {}", rec.id);
                println!("Files: {}  Bytes: {}  Hash: {}", rec.file_count, rec.total_bytes, rec.root_hash);
            }
            ReceiptCmd::VerifyNoDeletion { before, after } => {
                let before_str = std::fs::read_to_string(&before)?;
                let after_str  = std::fs::read_to_string(&after)?;
                let before_rec: cpmp::models::ScanReceipt = toml::from_str(&before_str)
                    .map_err(|e| anyhow::anyhow!("Failed to parse before receipt: {e}"))?;
                let after_rec: cpmp::models::ScanReceipt  = toml::from_str(&after_str)
                    .map_err(|e| anyhow::anyhow!("Failed to parse after receipt: {e}"))?;
                let report = receipt::verify_no_deletion(&before_rec, &after_rec);
                println!("{}", serde_json::to_string_pretty(&report)?);
                if !report.pass {
                    bail!("No-deletion verification FAILED: {} files deleted", report.deleted_files.len());
                }
                println!("✅ No-deletion verified.");
            }
        },

        Commands::Enterprise { sub } => match sub {
            EnterpriseCmd::Doctor { catalog } => {
                println!("# cpmp Enterprise Doctor\n");

                // Check open-ontologies binary
                let oo_ok = std::process::Command::new("open-ontologies")
                    .arg("--help").output().is_ok();
                println!("{} open-ontologies binary: {}", if oo_ok { "✅" } else { "❌" },
                    if oo_ok { "found" } else { "NOT FOUND — install open-ontologies" });

                // Check catalog
                if let Some(cat) = catalog {
                    let checks = policy::run_policy_checks(&cat);
                    println!("\n## Policy Packs\n");
                    policy::print_policy_report(&checks);
                } else {
                    println!("⚠️  No --catalog provided; skipping policy checks.");
                }

                println!("\n## Enterprise Module Stubs");
                for module in &[
                    "cpmp-enterprise-auth",
                    "cpmp-enterprise-tenancy",
                    "cpmp-enterprise-policy",
                    "cpmp-enterprise-audit",
                    "cpmp-enterprise-retention",
                    "cpmp-enterprise-backup",
                    "cpmp-enterprise-redaction",
                    "cpmp-enterprise-approval",
                    "cpmp-enterprise-observability",
                    "cpmp-open-ontologies-adapter",
                    "cpmp-ggen-projection",
                    "cpmp-public-vocabulary-firewall",
                ] {
                    println!("  ➖ {module} [stub — see docs/enterprise/]");
                }
            }
        },
    }

    Ok(())
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn default_cpmp_dir() -> PathBuf {
    dirs_home().join(".cpmp")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

/// Lightweight file walk to build FileEntry list (used for standalone receipt emit).
fn scan_to_entries(root: &PathBuf) -> Result<Vec<cpmp::models::FileEntry>> {
    use walkdir::WalkDir;
    let mut entries = Vec::new();
    for e in WalkDir::new(root).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if !e.file_type().is_file() { continue; }
        let path = e.path().to_path_buf();
        let bytes = std::fs::read(&path)?;
        let hash  = blake3::hash(&bytes).to_hex().to_string();
        let size  = e.metadata().map_or(0, |m| m.len());
        let lang  = path.extension().and_then(|x| x.to_str()).unwrap_or("").to_string();
        entries.push(cpmp::models::FileEntry { path, hash, size, language: lang });
    }
    Ok(entries)
}
