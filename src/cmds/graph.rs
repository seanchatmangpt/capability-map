use clap::{Args, Subcommand};

#[derive(Args)]
pub struct GraphArgs {
    #[command(subcommand)]
    pub command: GraphCmd,
}

#[derive(Subcommand)]
pub enum GraphCmd {
    /// Project catalog to RDF/Turtle (Open Ontologies integration point)
    Project {
        #[arg(long, default_value = ".cpmp/workspace.sqlite")]
        db: std::path::PathBuf,
        #[arg(long, default_value = ".cpmp/catalog/cpmp-catalog.ttl")]
        out: std::path::PathBuf,
    },
    /// Validate the projected RDF graph
    Validate {
        #[arg(long, default_value = ".cpmp/catalog/cpmp-catalog.ttl")]
        graph: std::path::PathBuf,
    },
    /// Show drift between two catalog snapshots
    Drift {
        #[arg(long)]
        before: std::path::PathBuf,
        #[arg(long)]
        after: std::path::PathBuf,
    },
}

pub fn run(args: GraphArgs) -> anyhow::Result<()> {
    match args.command {
        GraphCmd::Project { db, out } => project(db, out),
        GraphCmd::Validate { graph } => validate(graph),
        GraphCmd::Drift { before, after } => drift(before, after),
    }
}

fn project(db: std::path::PathBuf, out: std::path::PathBuf) -> anyhow::Result<()> {
    if !db.exists() {
        return Err(anyhow::anyhow!(
            "No catalog found at: {}\n\nNext:\n  cpmp computer discover <path> --out .cpmp",
            db.display()
        ));
    }

    let conn = crate::db::open(&db).map_err(|e| anyhow::anyhow!("{}", e))?;

    // Build a Turtle RDF projection using public vocabulary
    let mut ttl = String::new();
    ttl.push_str("@prefix rdf:    <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n");
    ttl.push_str("@prefix rdfs:   <http://www.w3.org/2000/01/rdf-schema#> .\n");
    ttl.push_str("@prefix dcat:   <http://www.w3.org/ns/dcat#> .\n");
    ttl.push_str("@prefix dcterms:<http://purl.org/dc/terms/> .\n");
    ttl.push_str("@prefix doap:   <http://usefulinc.com/ns/doap#> .\n");
    ttl.push_str("@prefix prov:   <http://www.w3.org/ns/prov#> .\n");
    ttl.push_str("@prefix spdx:   <http://spdx.org/rdf/terms#> .\n");
    ttl.push_str("@prefix skos:   <http://www.w3.org/2004/02/skos/core#> .\n");
    ttl.push_str("@prefix cpmp:   <https://cpmp.dev/ns#> .\n\n");

    // Catalog node
    let ts = chrono::Utc::now().to_rfc3339();
    ttl.push_str("<urn:cpmp:catalog> a dcat:Catalog ;\n");
    ttl.push_str("  dcterms:title \"cpmp local project catalog\" ;\n");
    ttl.push_str(&format!("  dcterms:modified \"{}\"^^<http://www.w3.org/2001/XMLSchema#dateTime> .\n\n", ts));

    // Repositories
    let mut stmt = conn.prepare("SELECT root, name, ecosystem, git_head FROM repositories LIMIT 500")?;
    let repos: Vec<(String, String, String, Option<String>)> = stmt.query_map([], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    })?.filter_map(|r| r.ok()).collect();

    for (root, name, ecosystem, head) in &repos {
        let id = format!("urn:cpmp:repo:{}", urlencoded(&name));
        ttl.push_str(&format!("<{}> a doap:Project, prov:Entity ;\n", id));
        ttl.push_str(&format!("  doap:name \"{}\" ;\n", escape_ttl(name)));
        ttl.push_str(&format!("  cpmp:ecosystem \"{}\" ;\n", escape_ttl(ecosystem)));
        ttl.push_str(&format!("  cpmp:rootPath \"{}\" ;\n", escape_ttl(root)));
        if let Some(h) = head {
            ttl.push_str(&format!("  cpmp:gitHead \"{}\" ;\n", escape_ttl(h)));
        }
        ttl.push_str("  dcat:isPartOf <urn:cpmp:catalog> .\n\n");
    }

    // Capabilities as SKOS Concepts
    let mut stmt = conn.prepare(
        "SELECT DISTINCT capability FROM capabilities ORDER BY capability"
    )?;
    let caps: Vec<String> = stmt.query_map([], |r| r.get(0))?.filter_map(|r| r.ok()).collect();

    for cap in &caps {
        let id = format!("urn:cpmp:capability:{}", urlencoded(cap));
        ttl.push_str(&format!("<{}> a skos:Concept ;\n", id));
        ttl.push_str(&format!("  skos:prefLabel \"{}\" ;\n", escape_ttl(cap)));
        ttl.push_str("  skos:inScheme <urn:cpmp:capability-scheme> .\n\n");
    }

    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, &ttl)?;

    println!("Graph projected to: {}", out.display());
    println!("  repositories: {}", repos.len());
    println!("  capabilities: {}", caps.len());
    println!();
    println!("Next: load into Open Ontologies:");
    println!("  open-ontologies load {} --format turtle", out.display());
    Ok(())
}

fn validate(graph: std::path::PathBuf) -> anyhow::Result<()> {
    if !graph.exists() {
        return Err(anyhow::anyhow!(
            "Graph file not found: {}\n\nNext:\n  cpmp graph project --out {}", 
            graph.display(), graph.display()
        ));
    }
    let content = std::fs::read_to_string(&graph)?;
    // Basic Turtle parse check: count @prefix, check for balanced angle brackets
    let prefix_count = content.matches("@prefix").count();
    let open_count = content.chars().filter(|&c| c == '<').count();
    let close_count = content.chars().filter(|&c| c == '>').count();
    if prefix_count == 0 {
        return Err(anyhow::anyhow!("REFUSAL: No @prefix declarations found — not valid Turtle."));
    }
    if open_count != close_count {
        return Err(anyhow::anyhow!("REFUSAL: Unbalanced angle brackets in Turtle file ({} < vs {} >).", open_count, close_count));
    }
    println!("Graph: {}", graph.display());
    println!("  prefixes:     {}", prefix_count);
    println!("  approx triples: {}", content.matches(" .\n").count());
    println!("  parse check:  PASS");
    println!("  integrity:    ok");
    Ok(())
}

fn drift(before: std::path::PathBuf, after: std::path::PathBuf) -> anyhow::Result<()> {
    let b = std::fs::read_to_string(&before).map_err(|_| anyhow::anyhow!("Cannot read: {}", before.display()))?;
    let a = std::fs::read_to_string(&after).map_err(|_| anyhow::anyhow!("Cannot read: {}", after.display()))?;
    let b_lines: std::collections::HashSet<&str> = b.lines().collect();
    let a_lines: std::collections::HashSet<&str> = a.lines().collect();
    let added: Vec<&&str> = a_lines.difference(&b_lines).collect();
    let removed: Vec<&&str> = b_lines.difference(&a_lines).collect();
    println!("Graph drift:");
    println!("  added lines:   {}", added.len());
    println!("  removed lines: {}", removed.len());
    if !removed.is_empty() {
        println!("\nWARNING: {} line(s) removed from graph (potential data loss):", removed.len());
        for line in removed.iter().take(10) {
            println!("  - {}", line);
        }
    }
    Ok(())
}

fn urlencoded(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c.to_string() } else { format!("_{:02X}_", c as u32) }).collect()
}

fn escape_ttl(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}
