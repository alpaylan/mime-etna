//! Fault-localization integration tests for mime.

use mime::etna::{
    property_ows_before_semicolon, property_quoted_vs_unquoted_param_eq,
    property_strip_empty_params, property_subtype_with_plus, PropertyResult,
};

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn prop_subtype_with_plus((a, b, c): (usize, usize, usize)) -> Option<bool> {
    to_opt(property_subtype_with_plus(a as u8, b as u8, c as u8))
}

fn prop_strip_empty_params((a, b, c): (usize, usize, usize)) -> Option<bool> {
    to_opt(property_strip_empty_params(a as u8, b as u8, c as u8))
}

fn prop_ows_before_semicolon(
    (a, (b, c, d)): (usize, (usize, usize, usize)),
) -> Option<bool> {
    to_opt(property_ows_before_semicolon(
        a as u8, b as u8, c as u8, d as u8,
    ))
}

fn prop_quoted_vs_unquoted_param_eq(
    (a, (b, c, d)): (usize, (usize, usize, usize)),
) -> Option<bool> {
    to_opt(property_quoted_vs_unquoted_param_eq(
        a as u8, b as u8, c as u8, d as u8,
    ))
}

fn emit_locate_json(r: &crabcheck::profiling::LocateResult) {
    use crabcheck::quickcheck::ResultStatus;
    let status = match &r.run.status {
        ResultStatus::Failed { .. } => "Failed",
        ResultStatus::Finished => "Finished",
        ResultStatus::GaveUp => "GaveUp",
        ResultStatus::TimedOut => "TimedOut",
        ResultStatus::Aborted { .. } => "Aborted",
    };
    let top = if let Some(s) = r.top() {
        serde_json::json!({
            "rank": s.rank,
            "file": s.region.file,
            "function": s.region.function,
            "start_line": s.region.start_line,
            "end_line": s.region.end_line,
            "ochiai": s.region.suspiciousness.ochiai,
            "delta": s.region.delta,
            "panic_overlap": s.panic_overlap,
            "confidence": format!("{}", s.confidence),
            "confidence_rule": s.confidence_rule,
        })
    } else {
        serde_json::Value::Null
    };
    let top_5: Vec<_> = r
        .suspects
        .iter()
        .take(5)
        .map(|s| {
            serde_json::json!({
                "rank": s.rank,
                "file": s.region.file,
                "function": s.region.function,
                "start_line": s.region.start_line,
                "end_line": s.region.end_line,
                "confidence": format!("{}", s.confidence),
                "confidence_rule": s.confidence_rule,
                "panic_overlap": s.panic_overlap,
            })
        })
        .collect();
    let diags: Vec<_> = r.diagnostics.iter().map(|d| d.tag()).collect();
    let out = serde_json::json!({
        "status": status,
        "passed": r.run.passed,
        "discarded": r.run.discarded,
        "n_panics": r.n_panics,
        "n_suspects": r.suspects.len(),
        "top": top,
        "top_5": top_5,
        "diagnostics": diags,
    });
    println!("@@LOCATE@@ {}", out);
}

#[test]
fn locate_subtype_with_plus() {
    let report = crabcheck::quickcheck_with_locate!(prop_subtype_with_plus, "mime");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_strip_empty_params() {
    let report = crabcheck::quickcheck_with_locate!(prop_strip_empty_params, "mime");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_ows_before_semicolon() {
    let report = crabcheck::quickcheck_with_locate!(prop_ows_before_semicolon, "mime");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_quoted_vs_unquoted_param_eq() {
    let report = crabcheck::quickcheck_with_locate!(prop_quoted_vs_unquoted_param_eq, "mime");
    eprintln!("{report}");
    emit_locate_json(&report);
}
