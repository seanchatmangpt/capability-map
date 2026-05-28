use std::path::PathBuf;
use anyhow::Result;
use crate::models::{FileEntry, DetectedCapability, Receipt, Symbol};
pub fn generate_reports(out: &PathBuf, files: &[FileEntry], _symbols: &[Symbol], caps: &[DetectedCapability]) -> Result<()> {
    std::fs::create_dir_all(out.join("reports"))?;
    std::fs::write(out.join("reports/CAPABILITY_INVENTORY.md"), "# Capability Inventory")?;
    std::fs::write(out.join("reports/PROJECT_ATLAS.md"), "# Project Atlas")?;
    Ok(())
}
pub fn generate_rdf_fallback(files: &[FileEntry], caps: &[DetectedCapability], receipt: &Receipt, out: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(out.join("catalog"))?;
    let ttl = format!("@prefix prov: <http://www.w3.org/ns/prov#> .\n<urn:cpmp:scan:{}> a prov:Activity .", receipt.id);
    std::fs::write(out.join("catalog/cpmp-catalog.ttl"), ttl)?;
    Ok(())
}
