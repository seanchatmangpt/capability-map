use crate::rdf::catalog_source_hash;
use crate::models::ScanReceipt;
use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;

/// Run `open-ontologies validate <ttl_path>`.
/// Gate: RDF must parse cleanly before load.
pub fn gate_validate(ttl_path: &Path) -> Result<()> {
    let status = Command::new("open-ontologies")
        .arg("validate")
        .arg(ttl_path)
        .status()?;
    if !status.success() {
        bail!("Gate[RDF_PARSE]: open-ontologies validate failed for {:?}", ttl_path);
    }
    Ok(())
}

/// Run `open-ontologies load <ttl_path>`.
pub fn gate_load(ttl_path: &Path) -> Result<()> {
    let status = Command::new("open-ontologies")
        .arg("load")
        .arg(ttl_path)
        .status()?;
    if !status.success() {
        bail!("Gate[LOAD]: open-ontologies load failed for {:?}", ttl_path);
    }
    Ok(())
}

/// Run `open-ontologies shacl <shapes_path>`.
pub fn gate_shacl(shapes_path: &Path) -> Result<()> {
    let status = Command::new("open-ontologies")
        .arg("shacl")
        .arg(shapes_path)
        .status()?;
    if !status.success() {
        bail!("Gate[SHACL]: SHACL validation failed for {:?}", shapes_path);
    }
    Ok(())
}

/// Run `open-ontologies version <version_label>`.
pub fn gate_version(version_label: &str) -> Result<()> {
    let status = Command::new("open-ontologies")
        .arg("version")
        .arg(version_label)
        .status()?;
    if !status.success() {
        bail!("Gate[VERSION]: open-ontologies version failed for label '{version_label}'");
    }
    Ok(())
}

/// Full admission pipeline: validate → load → shacl → version → emit receipt hash.
/// Returns the catalog source hash bound to this admission.
pub fn run_admission_gates(
    catalog_dir: &Path,
    receipt: &ScanReceipt,
) -> Result<String> {
    let ttl_path    = catalog_dir.join("cpmp-catalog.ttl");
    let shapes_path = catalog_dir.join("cpmp-shapes.ttl");
    let version_label = format!("v-{}", receipt.id);

    // Gate 1: RDF parses cleanly
    gate_validate(&ttl_path)?;

    // Gate 2: Load into Open Ontologies store
    gate_load(&ttl_path)?;

    // Gate 3: SHACL shapes pass
    gate_shacl(&shapes_path)?;

    // Gate 4: Version the graph
    gate_version(&version_label)?;

    // Gate 5: Bind report to catalog source hash
    let hash = catalog_source_hash(&ttl_path)?;

    eprintln!("Admission gates passed. Catalog hash: {hash}");
    Ok(hash)
}
