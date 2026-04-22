# mime — Injected Bugs

ETNA workload for the Rust `mime` crate (hyperium/mime). Each variant
re-introduces one historical bug-fix into a fresh patched branch and pairs
it with a framework-neutral property, four PBT adapters, and a deterministic
witness test.

Total mutations: 4

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `ows_before_semicolon_2e0268e_1` | `ows_before_semicolon` | `mime-parse/src/rfc7231.rs:99` | `patch` | `2e0268e6dc2933db308e452787c4ca85bc63bd6e` |
| 2 | `quoted_vs_unquoted_param_eq_0bba696_1` | `quoted_vs_unquoted_param_eq` | `src/cmp.rs:41` | `patch` | `0bba696c3795daaef945b821558cb2898a214ef1` |
| 3 | `strip_empty_params_7a39824_1` | `strip_empty_params` | `mime-parse/src/rfc7231.rs:144` | `marauders` | `7a39824f8eb816496895593ccda418c177ef5756` |
| 4 | `subtype_with_plus_5ebf32e_1` | `subtype_with_plus` | `mime-parse/src/lib.rs:119` | `marauders` | `5ebf32ed61c2642a295535e0d64c2f2c2ad1d1d2` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `ows_before_semicolon_2e0268e_1` | `OwsBeforeSemicolon` | `witness_ows_before_semicolon_case_text_plain_charset`, `witness_ows_before_semicolon_case_html_name_foo` |
| `quoted_vs_unquoted_param_eq_0bba696_1` | `QuotedVsUnquotedParamEq` | `witness_quoted_vs_unquoted_param_eq_case_name_foo`, `witness_quoted_vs_unquoted_param_eq_case_profile_bar` |
| `strip_empty_params_7a39824_1` | `StripEmptyParams` | `witness_strip_empty_params_case_event_stream_semicolon`, `witness_strip_empty_params_case_json_trailing_ws` |
| `subtype_with_plus_5ebf32e_1` | `SubtypeWithPlus` | `witness_subtype_with_plus_case_xhtml_xml`, `witness_subtype_with_plus_case_app_json_cbor` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `OwsBeforeSemicolon` | ✓ | ✓ | ✓ | ✓ |
| `QuotedVsUnquotedParamEq` | ✓ | ✓ | ✓ | ✓ |
| `StripEmptyParams` | ✓ | ✓ | ✓ | ✓ |
| `SubtypeWithPlus` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. ows_before_semicolon

- **Variant**: `ows_before_semicolon_2e0268e_1`
- **Location**: `mime-parse/src/rfc7231.rs:99`
- **Property**: `OwsBeforeSemicolon`
- **Witness(es)**:
  - `witness_ows_before_semicolon_case_text_plain_charset`
  - `witness_ows_before_semicolon_case_html_name_foo`
- **Source**: fix parsing OWS around parameters
  > RFC 7231 allows OWS between the subtype and the `;` introducing parameters; without the OWS arm in the sublevel loop, `a/b ; k=v` was rejected as an invalid token even though `a/b; k=v` parsed cleanly.
- **Fix commit**: `2e0268e6dc2933db308e452787c4ca85bc63bd6e` — fix parsing OWS around parameters
- **Invariant violated**: per RFC 7231, `media-type = type "/" subtype *( OWS ";" OWS parameter )`. The parser must accept optional whitespace between the subtype and the `;` that introduces the parameter list, and produce the same `MediaType` as the zero-whitespace form.
- **How the mutation triggers**: the fix extended the sublevel-parsing loop with an OWS match arm (`Some((i, b' ')) if i > start => { start = i; break; }`), so a space after the subtype is treated as the end of the subtype and parsing proceeds into the parameter loop. The variant (a `git apply` of `patches/ows_before_semicolon_2e0268e_1.patch`) deletes that arm. With the arm gone, the space hits the fall-through `Some((pos, byte)) => Err(ParseError::InvalidToken { ... })` branch and parsing fails. The property compares `parse("a/b; k=v")` with `parse("a/b ; k=v")` — the second `parse` returns `Err` under the variant, which is a `Fail`.

### 2. quoted_vs_unquoted_param_eq

- **Variant**: `quoted_vs_unquoted_param_eq_0bba696_1`
- **Location**: `src/cmp.rs:41`
- **Property**: `QuotedVsUnquotedParamEq`
- **Witness(es)**:
  - `witness_quoted_vs_unquoted_param_eq_case_name_foo`
  - `witness_quoted_vs_unquoted_param_eq_case_profile_bar`
- **Source**: fix comparison of quoted parameters
  > `params_eq` compared parameter values as raw `&str`, so `a/b; x=foo` and `a/b; x="foo"` compared unequal because one was three bytes and the other five; the fix routes comparisons through `Value`, which strips surrounding quotes before equality.
- **Fix commit**: `0bba696c3795daaef945b821558cb2898a214ef1` — fix comparison of quoted parameters
- **Invariant violated**: a quoted parameter value and an unquoted value with the same decoded content must compare equal. `parse("a/b; x=foo") == parse("a/b; x=\"foo\"")` must hold.
- **How the mutation triggers**: the fix introduced `src/cmp.rs::params_eq`, which iterates over `crate::value::params(a)` (yielding `Value` wrappers) and looks each parameter up via `crate::value::param(b, name)` — also returning `Value`. `Value::PartialEq` strips the surrounding quotes from the raw slice before comparing, so the two parses compare equal. The variant (a `git apply` of `patches/quoted_vs_unquoted_param_eq_0bba696_1.patch`) swaps those calls for `a.params()` and `b.param(name)` directly on `Mime`, which return raw `&str`. Under raw `&str` equality, `"foo"` (3 bytes) and `"\"foo\""` (5 bytes including quotes) are unequal, so `params_eq` returns `false` and the two parses compare unequal.

### 3. strip_empty_params

- **Variant**: `strip_empty_params_7a39824_1`
- **Location**: `mime-parse/src/rfc7231.rs:144`
- **Property**: `StripEmptyParams`
- **Witness(es)**:
  - `witness_strip_empty_params_case_event_stream_semicolon`
  - `witness_strip_empty_params_case_json_trailing_ws`
- **Source**: Strip empty parameter lists when parsing
  > When parsing `text/event-stream;` (trailing semicolon, no actual parameters), the parser interned the whole source including the `;`; the fix truncates the interned slice to `&s[..start]` so `has_params()` is `false` and the parse matches `text/event-stream`.
- **Fix commit**: `7a39824f8eb816496895593ccda418c177ef5756` — Strip empty parameter lists when parsing
- **Invariant violated**: when parsing succeeds and `ParamSource::None` is reported by the parameter sub-parser (i.e. there was a `;` but no actual parameters after it, possibly followed by whitespace), the interned source must be chopped at that `;`. The resulting mime must satisfy `has_params() == false`, `subtype()` must not include the trailing `;`, and `MediaType::parse("a/b;") == MediaType::parse("a/b")`.
- **How the mutation triggers**: the base arm for `ParamSource::None` in the source-building match is `Atoms::intern(&s[..start], slash, InternParams::None)`, where `start` points at the first byte of the empty param list. The variant replaces the arm with `Atoms::intern(s, slash, InternParams::None)` — the pre-7a39824 behaviour that keeps the trailing `;` (and any OWS) in the stored source. `subtype()`, which relies on `semicolon_or_end()` but for `ParamSource::None` falls through to `self.source.as_ref().len()`, now yields `"event-stream;"` on input `"text/event-stream;"`.

### 4. subtype_with_plus

- **Variant**: `subtype_with_plus_5ebf32e_1`
- **Location**: `mime-parse/src/lib.rs:119`
- **Property**: `SubtypeWithPlus`
- **Witness(es)**:
  - `witness_subtype_with_plus_case_xhtml_xml`
  - `witness_subtype_with_plus_case_app_json_cbor`
- **Source**: Fix subtype() to include the +suffix
  > `MediaType::subtype()` truncated at the `+` index, so `application/xhtml+xml` reported `"xhtml"` as the subtype; the fix returns the whole name (`"xhtml+xml"`) and exposes the suffix via a separate `suffix()` accessor.
- **Fix commit**: `5ebf32ed61c2642a295535e0d64c2f2c2ad1d1d2` — Fix subtype() to include the +suffix
- **Invariant violated**: `MediaType::subtype()` must return the entire `<name>+<suffix>` slice, not just the part before the first `+`.
- **How the mutation triggers**: the base computes the subtype end as `self.semicolon_or_end()`, i.e. the start of the parameter list (or end of source if there are no params). The variant replaces that with `self.plus.map(|p| p as usize).unwrap_or_else(|| self.semicolon_or_end())`, restoring the pre-fix behaviour that used the `+` index as the terminator. For any input of the form `<type>/<name>+<suffix>`, the variant's `subtype()` returns `<name>` instead of `<name>+<suffix>`. The property constructs `"<type>/<subtype>+<suffix>"` from the fixed pools and asserts both that `mt.subtype() == "<subtype>+<suffix>"` and that `mt.suffix() == Some(<suffix>)`; the first assertion is a `Fail` under the variant.
