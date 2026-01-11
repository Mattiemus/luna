use luna_lang::{Atom, BigFloat, BigInteger};
use luna_lang::{Context, Matcher};
use std::collections::HashMap;

#[test]
fn matcher_can_exact_match_string() {
    let context = Context::new("UnitTest");

    let matcher = Matcher::new(Atom::string("abc"), Atom::string("abc"), &context);

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_integer() {
    let context = Context::new("UnitTest");

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
    let context = Context::new("UnitTest");

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
    let context = Context::new("UnitTest");

    let matcher = Matcher::new(Atom::symbol("x"), Atom::symbol("x"), &context);

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_sexpression() {
    let context = Context::new("UnitTest");

    let matcher = Matcher::new(
        Atom::apply1(Atom::symbol("f"), Atom::symbol("x")),
        Atom::apply1(Atom::symbol("f"), Atom::symbol("x")),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_rejects_inexact_match() {
    let context = Context::new("UnitTest");

    let matcher = Matcher::new(Atom::symbol("x"), Atom::symbol("t"), &context);

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(0, matches.len());
}
