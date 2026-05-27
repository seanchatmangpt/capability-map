/// Enterprise policy gate results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GateResult {
    Pass,
    Fail { reason: String },
    Warning { message: String },
    Refusal { reason: String },
    NotApplicable,
}

impl std::fmt::Display for GateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateResult::Pass => write!(f, "PASS"),
            GateResult::Fail { reason } => write!(f, "FAIL: {reason}"),
            GateResult::Warning { message } => write!(f, "WARNING: {message}"),
            GateResult::Refusal { reason } => write!(f, "REFUSAL: {reason}"),
            GateResult::NotApplicable => write!(f, "N/A"),
        }
    }
}

/// A single policy check with result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyCheck {
    pub pack: String,
    pub result: GateResult,
}

/// Run all policy checks against the catalog directory.
pub fn run_policy_checks(catalog_dir: &std::path::Path) -> Vec<PolicyCheck> {
    let mut checks = Vec::new();

    // Pack: public-vocabulary-required
    let ttl_path = catalog_dir.join("cpmp-catalog.ttl");
    if ttl_path.exists() {
        let content = std::fs::read_to_string(&ttl_path).unwrap_or_default();
        // oxigraph 0.4 emits full IRIs without prefix declarations
        let has_prov = content.contains("http://www.w3.org/ns/prov#");
        let has_spdx = content.contains("http://spdx.org/rdf/terms#");
        let has_dcat = content.contains("http://www.w3.org/ns/dcat#");
        if has_prov && has_spdx && has_dcat {
            checks.push(PolicyCheck { pack: "public-vocabulary-required".into(), result: GateResult::Pass });
        } else {
            checks.push(PolicyCheck {
                pack: "public-vocabulary-required".into(),
                result: GateResult::Fail {
                    reason: format!(
                        "Missing vocabulary: prov={has_prov} spdx={has_spdx} dcat={has_dcat}"
                    ),
                },
            });
        }

        // Pack: no-private-predicate-authority
        let private_pred = content.contains("gall:") || content.contains("ggen:status");
        checks.push(PolicyCheck {
            pack: "no-private-predicate-authority".into(),
            result: if private_pred {
                GateResult::Fail { reason: "Private predicate authority detected".into() }
            } else {
                GateResult::Pass
            },
        });

        // Pack: prov-lineage-required
        let has_lineage = content.contains("wasGeneratedBy");
        checks.push(PolicyCheck {
            pack: "prov-lineage-required".into(),
            result: if has_lineage {
                GateResult::Pass
            } else {
                GateResult::Fail { reason: "Missing prov:wasGeneratedBy".into() }
            },
        });

        // Pack: spdx-checksum-required
        let has_checksum = content.contains("checksumValue") || content.contains("spdx:checksum");
        checks.push(PolicyCheck {
            pack: "spdx-checksum-required".into(),
            result: if has_checksum {
                GateResult::Pass
            } else {
                GateResult::Fail { reason: "Missing spdx:checksum triples".into() }
            },
        });
    } else {
        let r = GateResult::Refusal { reason: "cpmp-catalog.ttl not found".into() };
        for pack in &[
            "public-vocabulary-required",
            "no-private-predicate-authority",
            "prov-lineage-required",
            "spdx-checksum-required",
        ] {
            checks.push(PolicyCheck { pack: pack.to_string(), result: r.clone() });
        }
    }

    // Pack: no-deletion-required (check for verify-no-deletion receipts)
    let receipt_dir = catalog_dir.parent().and_then(|p| Some(p.join("receipts")));
    let has_receipts = receipt_dir
        .as_ref()
        .map_or(false, |d| d.exists() && std::fs::read_dir(d).map_or(false, |mut r| r.next().is_some()));
    checks.push(PolicyCheck {
        pack: "no-deletion-required".into(),
        result: if has_receipts {
            GateResult::Pass
        } else {
            GateResult::Warning { message: "No scan receipts found; cannot verify no-deletion".into() }
        },
    });

    // Pack: shacl-report-required
    let shapes_path = catalog_dir.join("cpmp-shapes.ttl");
    checks.push(PolicyCheck {
        pack: "shacl-report-required".into(),
        result: if shapes_path.exists() {
            GateResult::Pass
        } else {
            GateResult::Fail { reason: "cpmp-shapes.ttl not found".into() }
        },
    });

    checks
}

/// Print policy results and return exit code (0=all pass, 1=any fail/refusal).
pub fn print_policy_report(checks: &[PolicyCheck]) -> i32 {
    let mut exit = 0i32;
    for c in checks {
        let marker = match &c.result {
            GateResult::Pass => "✅",
            GateResult::Warning { .. } => "⚠️",
            GateResult::Fail { .. } | GateResult::Refusal { .. } => { exit = 1; "❌" }
            GateResult::NotApplicable => "➖",
        };
        println!("{marker} [{pack}] {result}", pack = c.pack, result = c.result);
    }
    exit
}
