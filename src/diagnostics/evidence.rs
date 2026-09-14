//! The Evidence output (Plan v3 D11): the Vocal Tract Lab Evidence tab's
//! layout — a NOT VALIDATED banner, a list of acceptance gates each with
//! its status, and a plain statement of what the software does and does
//! not establish. Built from what the runtime knows (self-tests,
//! provenance, calibration, the last validation report) and carried in
//! the diagnostics bundle; the Engineering Console renders it.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::core::SelfTestResult;
use crate::config::ValidationConfig;

pub const BANNER: &str = "NOT VALIDATED";
pub const BANNER_TEXT: &str =
    "Software consistency does not establish anatomical or clinical accuracy.";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Gate {
    pub name: String,
    pub status: String,
    /// `Some(true)` met, `Some(false)` not met, `None` not assessed.
    pub met: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub banner: String,
    pub banner_text: String,
    pub gates: Vec<Gate>,
    pub statement: String,
}

/// What the assembly draws on.
pub struct EvidenceInputs<'a> {
    pub self_tests: &'a [SelfTestResult],
    pub provenance: Option<&'a Value>,
    pub calibrated: bool,
    /// The last `vox-validation` report (`CompareReport` as JSON) and the
    /// pass verdict, if one was recorded.
    pub validation: Option<(&'a Value, bool)>,
    /// Phase 1 gate numbers from the last `pipeline_report` event.
    pub last_pipeline_report: Option<&'a std::collections::BTreeMap<String, String>>,
}

fn gate(name: &str, status: String, met: Option<bool>) -> Gate {
    Gate {
        name: name.into(),
        status,
        met,
    }
}

pub fn assemble(inputs: &EvidenceInputs<'_>) -> Evidence {
    let mut gates = Vec::new();

    // Kernel parity: the contract checks in the self-test.
    let contract: Vec<&SelfTestResult> = inputs
        .self_tests
        .iter()
        .filter(|t| t.code.starts_with("contract."))
        .collect();
    gates.push(if contract.is_empty() {
        gate(
            "Stages reproduce their kernels (contract checks)",
            "Not run in this session".into(),
            None,
        )
    } else {
        let passed = contract.iter().filter(|t| t.passed).count();
        gate(
            "Stages reproduce their kernels (contract checks)",
            format!("{passed} of {} passed · engineering", contract.len()),
            Some(passed == contract.len()),
        )
    });

    // Self-tests other than the contract checks.
    let other: Vec<&SelfTestResult> = inputs
        .self_tests
        .iter()
        .filter(|t| !t.code.starts_with("contract."))
        .collect();
    gates.push(if other.is_empty() {
        gate("Device self-test", "Not run in this session".into(), None)
    } else {
        let passed = other.iter().filter(|t| t.passed).count();
        gate(
            "Device self-test",
            format!("{passed} of {} passed", other.len()),
            Some(passed == other.len()),
        )
    });

    gates.push(match inputs.provenance {
        Some(p) => {
            let stages = p
                .get("stages")
                .and_then(Value::as_array)
                .map(Vec::len)
                .unwrap_or(0);
            let digest = p
                .get("params_sha256")
                .and_then(Value::as_str)
                .map(|d| d.chars().take(12).collect::<String>())
                .unwrap_or_default();
            gate(
                "Provenance recorded for the running pipeline",
                format!("{stages} stages · params {digest}"),
                Some(true),
            )
        }
        None => gate(
            "Provenance recorded for the running pipeline",
            "No pipeline built".into(),
            Some(false),
        ),
    });

    gates.push(match inputs.validation {
        Some((report, pass)) => {
            let hops = report
                .get("hops_compared")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            gate(
                "Provenance round-trip within tolerance bands",
                format!(
                    "{} · {hops} hops compared",
                    if pass {
                        "Within bands"
                    } else {
                        "Outside bands"
                    }
                ),
                Some(pass),
            )
        }
        None => gate(
            "Provenance round-trip within tolerance bands",
            "Not run (vox-validation)".into(),
            None,
        ),
    });

    gates.push(match inputs.last_pipeline_report {
        Some(r) => {
            let misses = r.get("misses").cloned().unwrap_or_else(|| "?".into());
            let worst = r.get("worst_hop_us").cloned().unwrap_or_else(|| "?".into());
            let budget = r.get("budget_us").cloned().unwrap_or_else(|| "?".into());
            let ok = misses == "0";
            gate(
                "Hop deadline (Phase 1 gate)",
                format!("misses {misses} · worst {worst} µs of {budget} µs"),
                Some(ok),
            )
        }
        None => gate(
            "Hop deadline (Phase 1 gate)",
            "No timing report yet".into(),
            None,
        ),
    });

    gates.push(gate(
        "Room calibration",
        if inputs.calibrated {
            "Completed · derived-only · model unchanged".into()
        } else {
            "Not completed in this session".into()
        },
        Some(inputs.calibrated),
    ));

    gates.push(gate(
        "Independent acoustic validation",
        "Required · not established".into(),
        Some(false),
    ));
    gates.push(gate(
        "Anatomical acceptance (Story two-mode fit)",
        "Not observed anatomy · a bounded model fit".into(),
        Some(false),
    ));

    // The atlas's own words, from the compiled-in data files (Phase 3).
    let files = crate::atlas::data_provenance();
    let model = files.first();
    let v = ValidationConfig::DEFAULT;
    gates.push(match model {
        Some(m) if m.model_id.is_empty() => gate(
            "Atlas data files load with their recorded digests",
            m.note.clone(),
            Some(false),
        ),
        Some(m) => gate(
            "Atlas data files load with their recorded digests",
            format!(
                "{} · {} · lumen {}",
                m.model_id,
                &m.sha256[..12],
                files.get(1).map(|l| &l.sha256[..12]).unwrap_or("?")
            ),
            Some(true),
        ),
        None => gate(
            "Atlas data files load with their recorded digests",
            "None compiled in".into(),
            Some(false),
        ),
    });
    gates.push(gate(
        "Atlas held-out surface error",
        format!(
            "median {:.3} mm reported · target ≤ {:.1} mm · {}",
            v.surface_error_reported_mm,
            v.surface_error_target_mm,
            if v.surface_error_reported_mm <= v.surface_error_target_mm {
                "met"
            } else {
                "not met"
            }
        ),
        Some(v.surface_error_reported_mm <= v.surface_error_target_mm),
    ));
    gates.push(gate(
        "Independent expert acceptances of the atlas",
        format!(
            "{}",
            model.map(|m| m.independent_expert_acceptances).unwrap_or(0)
        ),
        Some(model.is_some_and(|m| m.independent_expert_acceptances > 0)),
    ));
    gates.push(gate(
        "Scientific release readiness",
        format!(
            "{} (the atlas's own statement)",
            model.map(|m| m.scientific_release_ready).unwrap_or(false)
        ),
        Some(model.is_some_and(|m| m.scientific_release_ready)),
    ));

    Evidence {
        banner: BANNER.into(),
        banner_text: BANNER_TEXT.into(),
        gates,
        statement: "One pipeline throughout: the live readouts, the harness and the validation \
                    run the same stages from the same mode file. Targets and achieved values \
                    appear separately; a failed check is never relabeled as a pass."
            .into(),
    }
}

pub fn to_json(e: &Evidence) -> Value {
    serde_json::to_value(e).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gates_reflect_what_is_known() {
        let tests = vec![
            SelfTestResult {
                code: "contract.yin_matches_direct".into(),
                passed: true,
                message: String::new(),
            },
            SelfTestResult {
                code: "private_storage".into(),
                passed: false,
                message: String::new(),
            },
        ];
        let e = assemble(&EvidenceInputs {
            self_tests: &tests,
            provenance: None,
            calibrated: false,
            validation: None,
            last_pipeline_report: None,
        });
        assert_eq!(e.banner, BANNER);
        assert_eq!(e.gates[0].met, Some(true));
        assert_eq!(e.gates[1].met, Some(false));
        assert_eq!(e.gates[2].met, Some(false));
        assert_eq!(e.gates[3].met, None);
        let release = e
            .gates
            .iter()
            .find(|g| g.name == "Scientific release readiness")
            .unwrap();
        assert_eq!(release.met, Some(false));
        assert!(release.status.starts_with("false"));
        let surface = e
            .gates
            .iter()
            .find(|g| g.name == "Atlas held-out surface error")
            .unwrap();
        assert_eq!(surface.met, Some(false));
        assert!(surface.status.contains("4.781") && surface.status.ends_with("not met"));
        assert!(
            e.gates
                .iter()
                .any(|g| g.name.starts_with("Atlas data files") && g.met == Some(true))
        );
    }
}
