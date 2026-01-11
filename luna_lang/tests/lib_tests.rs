use luna_lang::abstractions::IString;
use luna_lang::{Atom, BigFloat, BigInteger, SExpression, Symbol};
use luna_lang::{Context, Matcher};
use std::collections::HashMap;

#[test]
fn matcher_can_exact_match_string() {
    let context = Context::new_global_context();

    let matcher = Matcher::new(
        Atom::String(IString::from("abc")),
        Atom::String(IString::from("abc")),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_integer() {
    let context = Context::new_global_context();

    let matcher = Matcher::new(
        Atom::Integer(BigInteger::from(123)),
        Atom::Integer(BigInteger::from(123)),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_real() {
    let context = Context::new_global_context();

    let matcher = Matcher::new(
        Atom::Real(BigFloat::with_val(32, 123.456)),
        Atom::Real(BigFloat::with_val(32, 123.456)),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_symbol() {
    let context = Context::new_global_context();

    let matcher = Matcher::new(Symbol::new("x"), Symbol::new("x"), &context);

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_sexpression() {
    let context = Context::new_global_context();

    let matcher = Matcher::new(
        SExpression::apply1(Symbol::new("f"), Symbol::new("x")),
        SExpression::apply1(Symbol::new("f"), Symbol::new("x")),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_rejects_inexact_match() {
    let context = Context::new_global_context();

    let matcher = Matcher::new(Symbol::new("x"), Symbol::new("t"), &context);

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(0, matches.len());
}
