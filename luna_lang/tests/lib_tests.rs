use luna_lang::normal::Normal;
use luna_lang::symbol::Symbol;
use luna_lang::{BigFloat, BigInteger, Expr};
use luna_lang::{Context, Matcher};
use std::collections::HashMap;

#[test]
fn matcher_can_exact_match_string() {
    let context = Context::new();

    let matcher = Matcher::new(Expr::from("abc"), Expr::from("abc"), &context);

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_integer() {
    let context = Context::new();

    let matcher = Matcher::new(
        Expr::from(BigInteger::from(123)),
        Expr::from(BigInteger::from(123)),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_real() {
    let context = Context::new();

    let matcher = Matcher::new(
        Expr::from(BigFloat::with_val(32, 123.456)),
        Expr::from(BigFloat::with_val(32, 123.456)),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_symbol() {
    let context = Context::new();

    let matcher = Matcher::new(
        Expr::from(Symbol::new("x")),
        Expr::from(Symbol::new("x")),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_can_exact_match_expression() {
    let context = Context::new();

    let matcher = Matcher::new(
        Expr::from(Normal::new(
            Expr::from(Symbol::new("f")),
            vec![Expr::from(Symbol::new("x"))],
        )),
        Expr::from(Normal::new(
            Expr::from(Symbol::new("f")),
            vec![Expr::from(Symbol::new("x"))],
        )),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(1, matches.len());
    assert_eq!(HashMap::new(), matches[0]);
}

#[test]
fn matcher_rejects_inexact_match() {
    let context = Context::new();

    let matcher = Matcher::new(
        Expr::from(Symbol::new("x")),
        Expr::from(Symbol::new("y")),
        &context,
    );

    let matches = matcher.collect::<Vec<_>>();

    assert_eq!(0, matches.len());
}
