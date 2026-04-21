//! Witness tests for ETNA variants.
//!
//! Each witness is a concrete `#[test]` that calls `property_<name>` with
//! frozen inputs. Passes on base HEAD. Fails when the corresponding
//! `etna/<variant>` branch is checked out (or when `M_<variant>=active`
//! activates a marauders mutation).

use mime::etna::{
    property_ows_before_semicolon, property_quoted_vs_unquoted_param_eq,
    property_strip_empty_params, property_subtype_with_plus, PropertyResult,
};

fn assert_pass(r: PropertyResult, label: &str) {
    match r {
        PropertyResult::Pass => {}
        PropertyResult::Discard => panic!("witness {}: property returned Discard", label),
        PropertyResult::Fail(m) => panic!("witness {}: property failed: {}", label, m),
    }
}

// ====== subtype_with_plus (commit 5ebf32e) ======
// Canonical input: "text/xhtml+xml". With the bug, subtype() returns
// "xhtml", truncating at the '+'. With the fix, subtype() returns
// "xhtml+xml".

#[test]
fn witness_subtype_with_plus_case_xhtml_xml() {
    // TYPES[0]="text", SUBTYPES[7]="xhtml", SUFFIXES[0]="xml"
    assert_pass(
        property_subtype_with_plus(0, 7, 0),
        "subtype_with_plus_case_xhtml_xml",
    );
}

#[test]
fn witness_subtype_with_plus_case_app_json_cbor() {
    // TYPES[1]="application", SUBTYPES[2]="json", SUFFIXES[2]="cbor"
    assert_pass(
        property_subtype_with_plus(1, 2, 2),
        "subtype_with_plus_case_app_json_cbor",
    );
}

// ====== strip_empty_params (commit 7a39824) ======
// Canonical input: "text/event-stream;" (and with trailing spaces).
// With the bug, the trailing ';' remains inside the mime's source so
// subtype() returns "event-stream;". With the fix, the source is rebuilt
// without the trailing ';'.

#[test]
fn witness_strip_empty_params_case_event_stream_semicolon() {
    // TYPES[0]="text", SUBTYPES[5]="event-stream", ws_count=0 -> "text/event-stream;"
    assert_pass(
        property_strip_empty_params(0, 5, 0),
        "strip_empty_params_case_event_stream_semicolon",
    );
}

#[test]
fn witness_strip_empty_params_case_json_trailing_ws() {
    // TYPES[1]="application", SUBTYPES[2]="json", ws_count=3 -> "application/json;   "
    assert_pass(
        property_strip_empty_params(1, 2, 3),
        "strip_empty_params_case_json_trailing_ws",
    );
}

// ====== ows_before_semicolon (commit 2e0268e) ======
// Canonical input: "text/plain ;charset=utf-8" (space between subtype and ';').
// With the bug, the parser emits InvalidToken at the space. With the fix,
// OWS is accepted and the mime is identical to the no-whitespace form.

#[test]
fn witness_ows_before_semicolon_case_text_plain_charset() {
    // TYPES[0]="text", SUBTYPES[0]="plain", PNAMES[0]="charset", PVALUES[4]="utf-8"
    assert_pass(
        property_ows_before_semicolon(0, 0, 0, 4),
        "ows_before_semicolon_case_text_plain_charset",
    );
}

#[test]
fn witness_ows_before_semicolon_case_html_name_foo() {
    // TYPES[0]="text", SUBTYPES[1]="html", PNAMES[3]="name", PVALUES[0]="foo"
    assert_pass(
        property_ows_before_semicolon(0, 1, 3, 0),
        "ows_before_semicolon_case_html_name_foo",
    );
}

// ====== quoted_vs_unquoted_param_eq (commit 0bba696) ======
// Canonical input: compare parse("text/plain; x=foo") with
// parse("text/plain; x=\"foo\""). With the bug, the raw &str values
// differ (lengths 3 vs 5) and the mimes are unequal. With the fix,
// Value::PartialEq uses quoted-string content equality.

#[test]
fn witness_quoted_vs_unquoted_param_eq_case_name_foo() {
    // TYPES[0]="text", SUBTYPES[0]="plain", PNAMES[3]="name", PVALUES[0]="foo"
    assert_pass(
        property_quoted_vs_unquoted_param_eq(0, 0, 3, 0),
        "quoted_vs_unquoted_param_eq_case_name_foo",
    );
}

#[test]
fn witness_quoted_vs_unquoted_param_eq_case_profile_bar() {
    // TYPES[1]="application", SUBTYPES[2]="json", PNAMES[2]="profile", PVALUES[1]="bar"
    assert_pass(
        property_quoted_vs_unquoted_param_eq(1, 2, 2, 1),
        "quoted_vs_unquoted_param_eq_case_profile_bar",
    );
}
