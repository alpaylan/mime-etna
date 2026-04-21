//! ETNA framework-neutral property functions for the `mime` crate.
//!
//! Each `property_<name>` is a pure function over concrete, owned inputs that
//! returns `PropertyResult`. The adapters in `src/bin/etna.rs` and the witness
//! tests in `tests/etna_witnesses.rs` both call these functions — the
//! invariant is never re-implemented inside an adapter.

#![allow(missing_docs)]

use crate::MediaType;

#[derive(Debug)]
pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

// Small fixed pools of valid token strings so generated inputs are always
// valid RFC 7231 type / subtype / param name / param value strings regardless
// of which PBT framework draws the indices.
const TYPES: &[&str] = &[
    "text",
    "application",
    "image",
    "audio",
    "video",
    "multipart",
];

const SUBTYPES: &[&str] = &[
    "plain",
    "html",
    "json",
    "css",
    "javascript",
    "event-stream",
    "form-data",
    "xhtml",
];

const SUFFIXES: &[&str] = &["xml", "json", "cbor", "zip"];

const PNAMES: &[&str] = &["charset", "boundary", "profile", "name"];

const PVALUES: &[&str] = &["foo", "bar", "abc123", "hello", "utf-8", "test"];

fn pick<'a>(pool: &'a [&'a str], i: u8) -> &'a str {
    pool[(i as usize) % pool.len()]
}

/// Subtype access must span the entire subtype range, including any
/// `+suffix`. The pre-fix bug (commit `5ebf32e`) used the plus index as the
/// subtype terminator, truncating the returned slice to the left-of-`+`
/// fragment.
///
/// Inputs:
///   - `t_idx`, `s_idx`, `suf_idx`: indices into the fixed TYPES / SUBTYPES /
///     SUFFIXES pools. Always produce a valid `<type>/<subtype>+<suffix>`
///     string, so the parse never fails and the invariant is always
///     evaluable.
pub fn property_subtype_with_plus(t_idx: u8, s_idx: u8, suf_idx: u8) -> PropertyResult {
    let t = pick(TYPES, t_idx);
    let s = pick(SUBTYPES, s_idx);
    let suf = pick(SUFFIXES, suf_idx);
    let input = format!("{}/{}+{}", t, s, suf);

    let mt = match MediaType::parse(input.as_str()) {
        Ok(m) => m,
        Err(_) => return PropertyResult::Discard,
    };

    let expected_subtype = format!("{}+{}", s, suf);
    let got_subtype = mt.subtype().to_string();
    if got_subtype != expected_subtype {
        return PropertyResult::Fail(format!(
            "subtype({:?}) = {:?}, expected {:?}",
            input, got_subtype, expected_subtype,
        ));
    }

    let got_suffix = mt.suffix().map(|s| s.to_string());
    if got_suffix.as_deref() != Some(suf) {
        return PropertyResult::Fail(format!(
            "suffix({:?}) = {:?}, expected Some({:?})",
            input, got_suffix, suf,
        ));
    }

    PropertyResult::Pass
}

/// An empty parameter list (a trailing `;` with only whitespace afterwards)
/// must be stripped at parse time. The fixed version of the parser
/// (commit `7a39824`) rebuilds the source without the trailing `;` so
/// `subtype()` does not include the semicolon and the resulting mime
/// compares equal to the same mime parsed without the trailing `;`.
///
/// Inputs:
///   - `t_idx`, `s_idx`: indices into TYPES / SUBTYPES pools.
///   - `ws_count`: how many trailing spaces to append after the `;`
///     (0..=4, clamped). Covers `"a/b;"`, `"a/b; "`, `"a/b;    "`.
pub fn property_strip_empty_params(t_idx: u8, s_idx: u8, ws_count: u8) -> PropertyResult {
    let t = pick(TYPES, t_idx);
    let s = pick(SUBTYPES, s_idx);
    let ws = (ws_count % 5) as usize;
    let mut with_semi = format!("{}/{};", t, s);
    for _ in 0..ws {
        with_semi.push(' ');
    }
    let without_semi = format!("{}/{}", t, s);

    let mt_with = match MediaType::parse(with_semi.as_str()) {
        Ok(m) => m,
        Err(e) => {
            return PropertyResult::Fail(format!(
                "parse({:?}) failed: {}",
                with_semi, e,
            ));
        }
    };
    let mt_without = match MediaType::parse(without_semi.as_str()) {
        Ok(m) => m,
        Err(_) => return PropertyResult::Discard,
    };

    if mt_with.has_params() {
        return PropertyResult::Fail(format!(
            "parse({:?}).has_params() = true, expected false",
            with_semi,
        ));
    }
    if mt_with.subtype() != s {
        return PropertyResult::Fail(format!(
            "parse({:?}).subtype() = {:?}, expected {:?}",
            with_semi,
            mt_with.subtype(),
            s,
        ));
    }
    if mt_with != mt_without {
        return PropertyResult::Fail(format!(
            "parse({:?}) != parse({:?})",
            with_semi, without_semi,
        ));
    }

    PropertyResult::Pass
}

/// Optional whitespace (OWS) between the subtype and the `;` that
/// introduces parameters must be accepted and produce the same mime as the
/// no-whitespace form. The pre-fix parser (`2e0268e`) rejected a space
/// between the subtype and `;` with InvalidToken.
///
/// Inputs:
///   - `t_idx`, `s_idx`, `p_idx`, `v_idx`: indices into the fixed pools.
///     Building a well-formed `<type>/<subtype>; <pname>=<pvalue>` string
///     so the well-formed parse always succeeds and we can compare.
pub fn property_ows_before_semicolon(
    t_idx: u8,
    s_idx: u8,
    p_idx: u8,
    v_idx: u8,
) -> PropertyResult {
    let t = pick(TYPES, t_idx);
    let s = pick(SUBTYPES, s_idx);
    let p = pick(PNAMES, p_idx);
    let v = pick(PVALUES, v_idx);

    let no_ws = format!("{}/{};{}={}", t, s, p, v);
    let with_ws = format!("{}/{} ;{}={}", t, s, p, v);

    let mt_no = match MediaType::parse(no_ws.as_str()) {
        Ok(m) => m,
        Err(_) => return PropertyResult::Discard,
    };
    let mt_ws = match MediaType::parse(with_ws.as_str()) {
        Ok(m) => m,
        Err(e) => {
            return PropertyResult::Fail(format!(
                "parse({:?}) failed: {}",
                with_ws, e,
            ));
        }
    };

    if mt_ws != mt_no {
        return PropertyResult::Fail(format!(
            "parse({:?}) != parse({:?})",
            with_ws, no_ws,
        ));
    }
    if mt_ws.param(p).map(|v| v.to_string()) != Some(v.to_string()) {
        return PropertyResult::Fail(format!(
            "parse({:?}).param({:?}) = {:?}, expected Some({:?})",
            with_ws,
            p,
            mt_ws.param(p).map(|v| v.to_string()),
            v,
        ));
    }

    PropertyResult::Pass
}

/// A quoted parameter value and an unquoted parameter value with the same
/// content must compare equal under `MediaType == MediaType`. The pre-fix
/// equality path (`0bba696`) compared raw `&str` param values, which treated
/// `"foo"` (three chars) and `"\"foo\""` (five chars) as unequal. The fix
/// wraps values in `Value`, whose `PartialEq` uses quoted-string content
/// equality.
///
/// Inputs:
///   - `t_idx`, `s_idx`, `p_idx`, `v_idx`: indices into the fixed pools.
///     Values in the pool are all simple tokens without escape-needing
///     characters so quoting does not alter their decoded value.
pub fn property_quoted_vs_unquoted_param_eq(
    t_idx: u8,
    s_idx: u8,
    p_idx: u8,
    v_idx: u8,
) -> PropertyResult {
    let t = pick(TYPES, t_idx);
    let s = pick(SUBTYPES, s_idx);
    let p = pick(PNAMES, p_idx);
    let v = pick(PVALUES, v_idx);

    let unquoted = format!("{}/{}; {}={}", t, s, p, v);
    let quoted = format!("{}/{}; {}=\"{}\"", t, s, p, v);

    let mt_u = match MediaType::parse(unquoted.as_str()) {
        Ok(m) => m,
        Err(_) => return PropertyResult::Discard,
    };
    let mt_q = match MediaType::parse(quoted.as_str()) {
        Ok(m) => m,
        Err(e) => {
            return PropertyResult::Fail(format!(
                "parse({:?}) failed: {}",
                quoted, e,
            ));
        }
    };

    if mt_u != mt_q {
        return PropertyResult::Fail(format!(
            "parse({:?}) != parse({:?})",
            unquoted, quoted,
        ));
    }

    PropertyResult::Pass
}
