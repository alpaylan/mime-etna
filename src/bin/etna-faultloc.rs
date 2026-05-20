use crabcheck::profiling::quickcheck;
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

// Crabcheck only has Mutate impls for (usize,usize,...), not (u8,u8,...).
// Properties take u8 byte indices into fixed pools, so any truncation
// preserves the distribution (the pools are small and modulo-indexed).

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        return;
    }
    let result = match (args[1].as_str(), args[2].as_str()) {
        ("crabcheck", "SubtypeWithPlus") => quickcheck(|(a, b, c): (usize, usize, usize)| {
            to_opt(property_subtype_with_plus(a as u8, b as u8, c as u8))
        }),
        ("crabcheck", "StripEmptyParams") => quickcheck(|(a, b, c): (usize, usize, usize)| {
            to_opt(property_strip_empty_params(a as u8, b as u8, c as u8))
        }),
        ("crabcheck", "OwsBeforeSemicolon") => {
            quickcheck(|(a, (b, c, d)): (usize, (usize, usize, usize))| {
                {
                    to_opt(property_ows_before_semicolon(
                        a as u8, b as u8, c as u8, d as u8,
                    ))
                }
            })
        }
        ("crabcheck", "QuotedVsUnquotedParamEq") => {
            quickcheck(|(a, (b, c, d)): (usize, (usize, usize, usize))| {
                {
                    to_opt(property_quoted_vs_unquoted_param_eq(
                        a as u8, b as u8, c as u8, d as u8,
                    ))
                }
            })
        }
        (a, b) => panic!("Unknown: {a} {b}"),
    };
    println!("Result: {:?}", result);
}
