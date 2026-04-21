// ETNA workload runner for mime.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: SubtypeWithPlus
//           | StripEmptyParams
//           | OwsBeforeSemicolon
//           | QuotedVsUnquotedParamEq
//           | All
//
// Every invocation prints exactly one JSON line to stdout and exits 0
// (except argv parsing, which exits 2). Etna reads status from JSON —
// not the exit code — so framework-level failures (counterexamples,
// timeouts) still produce exit 0.

use mime::etna::{
    property_ows_before_semicolon, property_quoted_vs_unquoted_param_eq,
    property_strip_empty_params, property_subtype_with_plus, PropertyResult,
};

use crabcheck::quickcheck as crabcheck_qc;
use hegel::{generators as hgen, HealthCheck, Hegel, Settings as HegelSettings, TestCase};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestError};
use quickcheck::{QuickCheck, ResultStatus, TestResult};

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "SubtypeWithPlus",
    "StripEmptyParams",
    "OwsBeforeSemicolon",
    "QuotedVsUnquotedParamEq",
];

fn cases_budget() -> u64 {
    std::env::var("ETNA_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200)
}

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    let mut final_status: Result<(), String> = Ok(());
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if r.is_err() && final_status.is_ok() {
            final_status = r;
        }
    }
    (final_status, total)
}

// ============================================================================
// Canonical witness inputs — keep in sync with tests/etna_witnesses.rs.
// ============================================================================

fn canonical_subtype_with_plus() -> (u8, u8, u8) {
    // witness_subtype_with_plus_case_xhtml_xml: text/xhtml+xml
    (0, 7, 0)
}

fn canonical_strip_empty_params() -> (u8, u8, u8) {
    // witness_strip_empty_params_case_event_stream_semicolon: text/event-stream;
    (0, 5, 0)
}

fn canonical_ows_before_semicolon() -> (u8, u8, u8, u8) {
    // witness_ows_before_semicolon_case_text_plain_charset: text/plain ;charset=utf-8
    (0, 0, 0, 4)
}

fn canonical_quoted_vs_unquoted() -> (u8, u8, u8, u8) {
    // witness_quoted_vs_unquoted_param_eq_case_name_foo: text/plain; name=foo vs "foo"
    (0, 0, 3, 0)
}

fn check_subtype_with_plus() -> Result<(), String> {
    let (a, b, c) = canonical_subtype_with_plus();
    to_err(property_subtype_with_plus(a, b, c))
}

fn check_strip_empty_params() -> Result<(), String> {
    let (a, b, c) = canonical_strip_empty_params();
    to_err(property_strip_empty_params(a, b, c))
}

fn check_ows_before_semicolon() -> Result<(), String> {
    let (a, b, c, d) = canonical_ows_before_semicolon();
    to_err(property_ows_before_semicolon(a, b, c, d))
}

fn check_quoted_vs_unquoted() -> Result<(), String> {
    let (a, b, c, d) = canonical_quoted_vs_unquoted();
    to_err(property_quoted_vs_unquoted_param_eq(a, b, c, d))
}

// ============================================================================
// etna tool — deterministic canonical replay.
// ============================================================================

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "SubtypeWithPlus" => check_subtype_with_plus(),
        "StripEmptyParams" => check_strip_empty_params(),
        "OwsBeforeSemicolon" => check_ows_before_semicolon(),
        "QuotedVsUnquotedParamEq" => check_quoted_vs_unquoted(),
        _ => {
            return (
                Err(format!("Unknown property for etna: {property}")),
                Metrics::default(),
            );
        }
    };
    (
        result,
        Metrics {
            inputs: 1,
            elapsed_us: t0.elapsed().as_micros(),
        },
    )
}

// ============================================================================
// proptest adapter
// ============================================================================

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let cfg = ProptestConfig {
        cases: cases_budget().min(u32::MAX as u64) as u32,
        max_shrink_iters: 32,
        failure_persistence: None,
        ..ProptestConfig::default()
    };
    let mut runner = proptest::test_runner::TestRunner::new(cfg);
    let result: Result<(), String> = match property {
        "SubtypeWithPlus" => {
            let c = counter.clone();
            runner
                .run(&(any::<u8>(), any::<u8>(), any::<u8>()), move |(a, b, cc)| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let cex = format!("(t_idx={} s_idx={} suf_idx={})", a, b, cc);
                    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        property_subtype_with_plus(a, b, cc)
                    }));
                    match out {
                        Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                        Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                    }
                })
                .map_err(|e| match e {
                    TestError::Fail(reason, _) => reason.to_string(),
                    other => other.to_string(),
                })
        }
        "StripEmptyParams" => {
            let c = counter.clone();
            runner
                .run(&(any::<u8>(), any::<u8>(), any::<u8>()), move |(a, b, cc)| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let cex = format!("(t_idx={} s_idx={} ws_count={})", a, b, cc);
                    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        property_strip_empty_params(a, b, cc)
                    }));
                    match out {
                        Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                        Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                    }
                })
                .map_err(|e| match e {
                    TestError::Fail(reason, _) => reason.to_string(),
                    other => other.to_string(),
                })
        }
        "OwsBeforeSemicolon" => {
            let c = counter.clone();
            runner
                .run(
                    &(any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>()),
                    move |(a, b, cc, d)| {
                        c.fetch_add(1, Ordering::Relaxed);
                        let cex = format!("(t_idx={} s_idx={} p_idx={} v_idx={})", a, b, cc, d);
                        let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            property_ows_before_semicolon(a, b, cc, d)
                        }));
                        match out {
                            Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                            Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                        }
                    },
                )
                .map_err(|e| match e {
                    TestError::Fail(reason, _) => reason.to_string(),
                    other => other.to_string(),
                })
        }
        "QuotedVsUnquotedParamEq" => {
            let c = counter.clone();
            runner
                .run(
                    &(any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>()),
                    move |(a, b, cc, d)| {
                        c.fetch_add(1, Ordering::Relaxed);
                        let cex = format!("(t_idx={} s_idx={} p_idx={} v_idx={})", a, b, cc, d);
                        let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            property_quoted_vs_unquoted_param_eq(a, b, cc, d)
                        }));
                        match out {
                            Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => Ok(()),
                            Ok(PropertyResult::Fail(_)) | Err(_) => Err(TestCaseError::fail(cex)),
                        }
                    },
                )
                .map_err(|e| match e {
                    TestError::Fail(reason, _) => reason.to_string(),
                    other => other.to_string(),
                })
        }
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// ============================================================================
// quickcheck adapter (fork with `etna` feature — fn-pointer API)
// ============================================================================

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_subtype_with_plus(a: u8, b: u8, c: u8) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        property_subtype_with_plus(a, b, c)
    }));
    match out {
        Ok(PropertyResult::Pass) => TestResult::passed(),
        Ok(PropertyResult::Discard) => TestResult::discard(),
        Ok(PropertyResult::Fail(_)) | Err(_) => TestResult::failed(),
    }
}

fn qc_strip_empty_params(a: u8, b: u8, c: u8) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        property_strip_empty_params(a, b, c)
    }));
    match out {
        Ok(PropertyResult::Pass) => TestResult::passed(),
        Ok(PropertyResult::Discard) => TestResult::discard(),
        Ok(PropertyResult::Fail(_)) | Err(_) => TestResult::failed(),
    }
}

fn qc_ows_before_semicolon(a: u8, b: u8, c: u8, d: u8) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        property_ows_before_semicolon(a, b, c, d)
    }));
    match out {
        Ok(PropertyResult::Pass) => TestResult::passed(),
        Ok(PropertyResult::Discard) => TestResult::discard(),
        Ok(PropertyResult::Fail(_)) | Err(_) => TestResult::failed(),
    }
}

fn qc_quoted_vs_unquoted(a: u8, b: u8, c: u8, d: u8) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        property_quoted_vs_unquoted_param_eq(a, b, c, d)
    }));
    match out {
        Ok(PropertyResult::Pass) => TestResult::passed(),
        Ok(PropertyResult::Discard) => TestResult::discard(),
        Ok(PropertyResult::Fail(_)) | Err(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let budget = cases_budget();
    let qc_builder = || {
        QuickCheck::new()
            .tests(budget)
            .max_tests(budget.saturating_mul(4))
            .max_time(Duration::from_secs(86_400))
    };
    let result = match property {
        "SubtypeWithPlus" => qc_builder().quicktest(qc_subtype_with_plus as fn(u8, u8, u8) -> TestResult),
        "StripEmptyParams" => qc_builder().quicktest(qc_strip_empty_params as fn(u8, u8, u8) -> TestResult),
        "OwsBeforeSemicolon" => {
            qc_builder().quicktest(qc_ows_before_semicolon as fn(u8, u8, u8, u8) -> TestResult)
        }
        "QuotedVsUnquotedParamEq" => {
            qc_builder().quicktest(qc_quoted_vs_unquoted as fn(u8, u8, u8, u8) -> TestResult)
        }
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!("({})", arguments.join(" "))),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {err:?}")),
        ResultStatus::TimedOut => Err("quickcheck timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up after {} tests",
            result.n_tests_passed
        )),
    };
    (status, Metrics { inputs, elapsed_us })
}

// ============================================================================
// crabcheck adapter (fn-pointer API)
// ============================================================================

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_subtype_with_plus(args: (u8, u8, u8)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_subtype_with_plus(args.0, args.1, args.2) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_strip_empty_params(args: (u8, u8, u8)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_strip_empty_params(args.0, args.1, args.2) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_ows_before_semicolon(args: (u8, u8, u8, u8)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_ows_before_semicolon(args.0, args.1, args.2, args.3) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_quoted_vs_unquoted(args: (u8, u8, u8, u8)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_quoted_vs_unquoted_param_eq(args.0, args.1, args.2, args.3) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let cc_config = crabcheck::quickcheck::Config {
        tests: cases_budget(),
    };
    let result = match property {
        "SubtypeWithPlus" => crabcheck::quickcheck::quickcheck_with_config(
            cc_config,
            cc_subtype_with_plus as fn((u8, u8, u8)) -> Option<bool>,
        ),
        "StripEmptyParams" => crabcheck::quickcheck::quickcheck_with_config(
            cc_config,
            cc_strip_empty_params as fn((u8, u8, u8)) -> Option<bool>,
        ),
        "OwsBeforeSemicolon" => crabcheck::quickcheck::quickcheck_with_config(
            cc_config,
            cc_ows_before_semicolon as fn((u8, u8, u8, u8)) -> Option<bool>,
        ),
        "QuotedVsUnquotedParamEq" => crabcheck::quickcheck::quickcheck_with_config(
            cc_config,
            cc_quoted_vs_unquoted as fn((u8, u8, u8, u8)) -> Option<bool>,
        ),
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        }
        crabcheck_qc::ResultStatus::TimedOut => Err("crabcheck timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {error}"))
        }
    };
    (status, Metrics { inputs, elapsed_us })
}

// ============================================================================
// hegel adapter (real hegeltest 0.3.7 — panic-on-cex API)
// ============================================================================

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new()
        .test_cases(cases_budget())
        .suppress_health_check(HealthCheck::all())
}

fn hg_draw_u8(tc: &TestCase) -> u8 {
    tc.draw(hgen::integers::<u64>().min_value(0).max_value(u8::MAX as u64)) as u8
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "SubtypeWithPlus" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a = hg_draw_u8(&tc);
                let b = hg_draw_u8(&tc);
                let cc = hg_draw_u8(&tc);
                let cex = format!("(t_idx={} s_idx={} suf_idx={})", a, b, cc);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_subtype_with_plus(a, b, cc)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "StripEmptyParams" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a = hg_draw_u8(&tc);
                let b = hg_draw_u8(&tc);
                let cc = hg_draw_u8(&tc);
                let cex = format!("(t_idx={} s_idx={} ws_count={})", a, b, cc);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_strip_empty_params(a, b, cc)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "OwsBeforeSemicolon" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a = hg_draw_u8(&tc);
                let b = hg_draw_u8(&tc);
                let cc = hg_draw_u8(&tc);
                let d = hg_draw_u8(&tc);
                let cex = format!("(t_idx={} s_idx={} p_idx={} v_idx={})", a, b, cc, d);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_ows_before_semicolon(a, b, cc, d)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        "QuotedVsUnquotedParamEq" => {
            Hegel::new(|tc: TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a = hg_draw_u8(&tc);
                let b = hg_draw_u8(&tc);
                let cc = hg_draw_u8(&tc);
                let d = hg_draw_u8(&tc);
                let cex = format!("(t_idx={} s_idx={} p_idx={} v_idx={})", a, b, cc, d);
                let out = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    property_quoted_vs_unquoted_param_eq(a, b, cc, d)
                }));
                match out {
                    Ok(PropertyResult::Pass) | Ok(PropertyResult::Discard) => {}
                    Ok(PropertyResult::Fail(_)) | Err(_) => panic!("{}", cex),
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{}", property),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(msg
                .strip_prefix("Property test failed: ")
                .unwrap_or(&msg)
                .to_string())
        }
    };
    (status, metrics)
}

// ============================================================================
// dispatch + main
// ============================================================================

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (Err(format!("Unknown tool: {tool}")), Metrics::default()),
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!(
            "Properties: SubtypeWithPlus | StripEmptyParams | OwsBeforeSemicolon | \
             QuotedVsUnquotedParamEq | All"
        );
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "panic with non-string payload".to_string()
            };
            emit_json(tool, property, "aborted", Metrics::default(), None, Some(&msg));
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(e) => emit_json(tool, property, "failed", metrics, Some(&e), None),
    }
}
