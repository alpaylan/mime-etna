# mime — ETNA Tasks

Total tasks: 16

ETNA tasks are **mutation/property/witness triplets**. Each row below is one
runnable task: check out the variant branch, run the command, and observe
the emitted JSON line on stdout.

## Task Index

| Task | Variant | Framework | Property | Witness | Command |
|------|---------|-----------|----------|---------|---------|
| 001  | `subtype_with_plus_5ebf32e_1` | proptest    | `property_subtype_with_plus` | `witness_subtype_with_plus_case_xhtml_xml` | `cargo run --release --bin etna -- proptest SubtypeWithPlus` |
| 002  | `subtype_with_plus_5ebf32e_1` | quickcheck  | `property_subtype_with_plus` | `witness_subtype_with_plus_case_xhtml_xml` | `cargo run --release --bin etna -- quickcheck SubtypeWithPlus` |
| 003  | `subtype_with_plus_5ebf32e_1` | crabcheck   | `property_subtype_with_plus` | `witness_subtype_with_plus_case_xhtml_xml` | `cargo run --release --bin etna -- crabcheck SubtypeWithPlus` |
| 004  | `subtype_with_plus_5ebf32e_1` | hegel       | `property_subtype_with_plus` | `witness_subtype_with_plus_case_xhtml_xml` | `cargo run --release --bin etna -- hegel SubtypeWithPlus` |
| 005  | `strip_empty_params_7a39824_1` | proptest    | `property_strip_empty_params` | `witness_strip_empty_params_case_event_stream_semicolon` | `cargo run --release --bin etna -- proptest StripEmptyParams` |
| 006  | `strip_empty_params_7a39824_1` | quickcheck  | `property_strip_empty_params` | `witness_strip_empty_params_case_event_stream_semicolon` | `cargo run --release --bin etna -- quickcheck StripEmptyParams` |
| 007  | `strip_empty_params_7a39824_1` | crabcheck   | `property_strip_empty_params` | `witness_strip_empty_params_case_event_stream_semicolon` | `cargo run --release --bin etna -- crabcheck StripEmptyParams` |
| 008  | `strip_empty_params_7a39824_1` | hegel       | `property_strip_empty_params` | `witness_strip_empty_params_case_event_stream_semicolon` | `cargo run --release --bin etna -- hegel StripEmptyParams` |
| 009  | `ows_before_semicolon_2e0268e_1` | proptest    | `property_ows_before_semicolon` | `witness_ows_before_semicolon_case_text_plain_charset` | `cargo run --release --bin etna -- proptest OwsBeforeSemicolon` |
| 010  | `ows_before_semicolon_2e0268e_1` | quickcheck  | `property_ows_before_semicolon` | `witness_ows_before_semicolon_case_text_plain_charset` | `cargo run --release --bin etna -- quickcheck OwsBeforeSemicolon` |
| 011  | `ows_before_semicolon_2e0268e_1` | crabcheck   | `property_ows_before_semicolon` | `witness_ows_before_semicolon_case_text_plain_charset` | `cargo run --release --bin etna -- crabcheck OwsBeforeSemicolon` |
| 012  | `ows_before_semicolon_2e0268e_1` | hegel       | `property_ows_before_semicolon` | `witness_ows_before_semicolon_case_text_plain_charset` | `cargo run --release --bin etna -- hegel OwsBeforeSemicolon` |
| 013  | `quoted_vs_unquoted_param_eq_0bba696_1` | proptest    | `property_quoted_vs_unquoted_param_eq` | `witness_quoted_vs_unquoted_param_eq_case_name_foo` | `cargo run --release --bin etna -- proptest QuotedVsUnquotedParamEq` |
| 014  | `quoted_vs_unquoted_param_eq_0bba696_1` | quickcheck  | `property_quoted_vs_unquoted_param_eq` | `witness_quoted_vs_unquoted_param_eq_case_name_foo` | `cargo run --release --bin etna -- quickcheck QuotedVsUnquotedParamEq` |
| 015  | `quoted_vs_unquoted_param_eq_0bba696_1` | crabcheck   | `property_quoted_vs_unquoted_param_eq` | `witness_quoted_vs_unquoted_param_eq_case_name_foo` | `cargo run --release --bin etna -- crabcheck QuotedVsUnquotedParamEq` |
| 016  | `quoted_vs_unquoted_param_eq_0bba696_1` | hegel       | `property_quoted_vs_unquoted_param_eq` | `witness_quoted_vs_unquoted_param_eq_case_name_foo` | `cargo run --release --bin etna -- hegel QuotedVsUnquotedParamEq` |

## Witness catalog

Each witness is a deterministic concrete test. Base build: passes. Variant-active build: fails.

- `witness_subtype_with_plus_case_xhtml_xml` — `TYPES[0]="text"`, `SUBTYPES[7]="xhtml"`, `SUFFIXES[0]="xml"` → base returns `"xhtml+xml"` from `MediaType::parse("text/xhtml+xml").subtype()`; variant returns `"xhtml"`.
- `witness_subtype_with_plus_case_app_json_cbor` — `TYPES[1]="application"`, `SUBTYPES[2]="json"`, `SUFFIXES[2]="cbor"` → base returns `"json+cbor"`; variant returns `"json"`. A second canonical case guards against a variant that happens to return the full subtype for one atom-interned mime and the truncated subtype for another.
- `witness_strip_empty_params_case_event_stream_semicolon` — `TYPES[0]="text"`, `SUBTYPES[5]="event-stream"`, `ws_count=0` → base parses `"text/event-stream;"` to a mime equal to `MediaType::parse("text/event-stream")`; variant keeps the trailing `;` in the source and returns `"event-stream;"` from `subtype()`.
- `witness_strip_empty_params_case_json_trailing_ws` — `TYPES[1]="application"`, `SUBTYPES[2]="json"`, `ws_count=3` → covers the `"application/json;   "` form where extra trailing OWS exercises the same chop-at-start logic in the fix.
- `witness_ows_before_semicolon_case_text_plain_charset` — `TYPES[0]="text"`, `SUBTYPES[0]="plain"`, `PNAMES[0]="charset"`, `PVALUES[4]="utf-8"` → base parses `"text/plain ;charset=utf-8"` into the same mime as `"text/plain;charset=utf-8"`; variant returns `Err(InvalidToken(' ' at position 10))`.
- `witness_ows_before_semicolon_case_html_name_foo` — `TYPES[0]="text"`, `SUBTYPES[1]="html"`, `PNAMES[3]="name"`, `PVALUES[0]="foo"` → exercises the same OWS-acceptance invariant for a different type/param combination.
- `witness_quoted_vs_unquoted_param_eq_case_name_foo` — `TYPES[0]="text"`, `SUBTYPES[0]="plain"`, `PNAMES[3]="name"`, `PVALUES[0]="foo"` → base says `parse("text/plain; name=foo") == parse("text/plain; name=\"foo\"")`; variant compares raw `&str` values, so `"foo"` (3 bytes) vs `"\"foo\""` (5 bytes) are unequal and the mimes compare unequal.
- `witness_quoted_vs_unquoted_param_eq_case_profile_bar` — `TYPES[1]="application"`, `SUBTYPES[2]="json"`, `PNAMES[2]="profile"`, `PVALUES[1]="bar"` → a second canonical case using a different atom (`application/json` is interned) so the `(atom, atom)` short-circuit in `mime_eq` is forced to the slow-path branch where raw-vs-`Value` comparison matters.

## Running a task

1. Make sure you're on the variant branch you want to reproduce.
   ```
   git checkout etna/subtype_with_plus_5ebf32e_1
   ```
   For a task row that targets the base behaviour, use `master` instead.

2. Build and run:
   ```
   cargo run --release --bin etna -- proptest SubtypeWithPlus
   ```

3. The runner always exits 0 and emits exactly one JSON line on stdout with
   the schema:
   ```
   {"status": "passed" | "failed" | "discarded" | "aborted",
    "tests": <usize>, "discards": <usize>, "time": "<duration>",
    "counterexample": <string|null>, "error": <string|null>,
    "tool": "<framework>", "property": "<PropertyName>"}
   ```
   On a variant branch the expected outcome for that variant's property is
   `status == "failed"` with a `counterexample` field populated. On `master`,
   every `status` should be `"passed"`.
