use luna_lang::Symbol;
use luna_lang::{Context, Matcher};
use std::collections::HashMap;

#[test]
fn matcher_can_exact_match() {
    let context = Context::new_global_context();

    let mut matcher = Matcher::new(
        Symbol::from_static_str("x"),
        Symbol::from_static_str("x"),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}
