mod symbols;

use crate::{
    Attribute, BigFloat, BigInteger, DEFAULT_REAL_PRECISION, EvalResult, SymbolValue, parse, sym,
    try_sequence,
};
use crate::{Context, Expr, SolutionSet};
use crate::{ExprKind, Symbol, extract_condition};
use crate::{Normal, ValueType};
use rug::ops::AddFrom;
use std::ops::{AddAssign, MulAssign};

pub use symbols::*;

pub type BuiltinFn = fn(SolutionSet, Expr, &Context) -> EvalResult;
pub type BuiltinFnMut = fn(SolutionSet, Expr, &mut Context) -> EvalResult;

/// Registers all builtins.
pub(crate) fn register_builtins(context: &mut Context) {
    register_set_builtin(context);
    register_set_delayed_builtin(context);
    register_head_builtin(context);
    register_plus_builtin(context);
    register_times_builtin(context);
    register_subtract_builtin(context);
}

/// Registers the `Set` builtin symbol.
///
/// - `Attributes[Set] = { ReadOnly, AttributesReadOnly, HoldFirst, HoldSequences }`
/// - `Set[lhs_, rhs_] := built-in`
pub(crate) fn register_set_builtin(context: &mut Context) {
    context
        .set_value(
            &sym!(Set),
            ValueType::DownValue,
            SymbolValue::BuiltInMut {
                pattern: parse!("Set[lhs_, rhs_]"),
                condition: None,
                built_in: |arguments, _, context| {
                    let pattern = &arguments[&Symbol::new("lhs")];
                    let ground = &arguments[&Symbol::new("rhs")];

                    declare_rule(pattern, ground, context);

                    EvalResult::Changed(Expr::from(Normal::new(sym!(Hold), vec![ground.clone()])))
                },
            },
        )
        .unwrap();

    context
        .set_attributes(
            &sym!(Set),
            Attribute::ReadOnly
                + Attribute::AttributesReadOnly
                + Attribute::HoldFirst
                + Attribute::HoldSequences,
        )
        .unwrap();
}

/// Registers the `SetDelayed` builtin symbol.
///
/// - `Attributes[SetDelayed] = { ReadOnly, AttributesReadOnly, HoldAll, HoldSequences }`
/// - `SetDelayed[lhs_, rhs_] := built-in`
pub(crate) fn register_set_delayed_builtin(context: &mut Context) {
    context
        .set_value(
            &sym!(SetDelayed),
            ValueType::DownValue,
            SymbolValue::BuiltInMut {
                pattern: parse!("SetDelayed[lhs_, rhs_]"),
                condition: None,
                built_in: |arguments, _, context| {
                    let pattern = &arguments[&Symbol::new("lhs")];
                    let ground = &arguments[&Symbol::new("rhs")];

                    declare_rule(pattern, ground, context);

                    EvalResult::Changed(Expr::from(sym!(Null)))
                },
            },
        )
        .unwrap();

    context
        .set_attributes(
            &sym!(SetDelayed),
            Attribute::ReadOnly
                + Attribute::AttributesReadOnly
                + Attribute::HoldAll
                + Attribute::HoldSequences,
        )
        .unwrap();
}

fn declare_rule(pattern: &Expr, ground: &Expr, context: &mut Context) {
    let (ground, condition) = extract_condition(ground);

    let value = SymbolValue::Definitions {
        pattern: pattern.clone(),
        condition: condition.cloned(),
        ground: ground.clone(),
    };

    let name = pattern.name().unwrap();

    match pattern.kind() {
        ExprKind::Symbol(_) => {
            context.set_value(name, ValueType::OwnValue, value).unwrap();
        }
        ExprKind::Normal(_) => {
            context
                .set_value(name, ValueType::DownValue, value)
                .unwrap();
        }
        _ => todo!(),
    }
}

/// Registers the `Head` builtin symbol.
///
/// - `Attributes[Head] = { ReadOnly, AttributesReadOnly }`
/// - `Head[expr_] := built-in`
/// - `Head[expr_, h_] := built-in`
pub(crate) fn register_head_builtin(context: &mut Context) {
    context
        .set_value(
            &sym!(Head),
            ValueType::DownValue,
            SymbolValue::BuiltIn {
                pattern: parse!("Head[expr_]"),
                condition: None,
                built_in: |arguments, _, _| {
                    let expr = &arguments[&Symbol::new("expr")];

                    EvalResult::Changed(expr.head().clone())
                },
            },
        )
        .unwrap();

    context
        .set_value(
            &sym!(Head),
            ValueType::DownValue,
            SymbolValue::BuiltIn {
                pattern: parse!("Head[expr_, h_]"),
                condition: None,
                built_in: |arguments, _, _| {
                    let expr = &arguments[&Symbol::new("expr")];
                    let h = &arguments[&Symbol::new("h")];

                    EvalResult::Changed(Expr::from(Normal::new(
                        h.clone(),
                        vec![expr.head().clone()],
                    )))
                },
            },
        )
        .unwrap();

    context
        .set_attributes(
            &sym!(Head),
            Attribute::ReadOnly + Attribute::AttributesReadOnly,
        )
        .unwrap();
}

/// Registers the `Plus` builtin symbol.
///
/// - `Attributes[Plus] = { ReadOnly, AttributesReadOnly, Associative, Commutative }`
/// - `Plus[exprs___] := built-in`
pub(crate) fn register_plus_builtin(context: &mut Context) {
    context
        .set_value(
            &sym!(Plus),
            ValueType::DownValue,
            SymbolValue::BuiltIn {
                pattern: parse!("Plus[exprs___]"),
                condition: None,
                built_in: |arguments, expr, _| {
                    let exprs = &arguments[&Symbol::new("exprs")];
                    let expr_elements =
                        try_sequence(exprs).expect("expected exprs___ to match Sequence[]");

                    let mut int_accumulator = BigInteger::new();
                    let mut real_accumulator = BigFloat::new(DEFAULT_REAL_PRECISION);
                    let mut seen_real = false;

                    let mut new_elements = Vec::with_capacity(expr_elements.len());

                    for expr in expr_elements {
                        match expr.kind() {
                            ExprKind::Integer(n) => {
                                int_accumulator.add_assign(n);
                            }
                            ExprKind::Real(r) => {
                                real_accumulator.add_assign(r.as_float());
                                seen_real = true;
                            }
                            _ => {
                                new_elements.push(expr.clone());
                            }
                        }
                    }

                    if seen_real {
                        real_accumulator.add_from(&int_accumulator);
                        new_elements.push(Expr::from(real_accumulator));
                    } else if int_accumulator != 0 {
                        new_elements.push(Expr::from(int_accumulator));
                    }

                    if new_elements.len() == 0 {
                        EvalResult::Changed(Expr::from(BigInteger::new()))
                    } else if new_elements.len() == 1 {
                        EvalResult::Changed(new_elements[0].clone())
                    } else if new_elements != expr_elements {
                        EvalResult::Changed(Expr::from(Normal::new(sym!(Plus), new_elements)))
                    } else {
                        EvalResult::Unchanged(expr)
                    }
                },
            },
        )
        .unwrap();

    context
        .set_attributes(
            &sym!(Plus),
            Attribute::ReadOnly
                + Attribute::AttributesReadOnly
                + Attribute::Associative
                + Attribute::Commutative,
        )
        .unwrap();
}

/// Registers the `Times` builtin symbol.
///
/// - `Attributes[Times] = { ReadOnly, AttributesReadOnly, Associative, Commutative }`
/// - `Times[exprs___] := built-in`
pub(crate) fn register_times_builtin(context: &mut Context) {
    context
        .set_value(
            &sym!(Times),
            ValueType::DownValue,
            SymbolValue::BuiltIn {
                pattern: parse!("Times[exprs___]"),
                condition: None,
                built_in: |arguments, expr, _| {
                    let exprs = &arguments[&Symbol::new("exprs")];
                    let expr_elements =
                        try_sequence(exprs).expect("expected exprs___ to match Sequence[]");

                    let mut int_accumulator = BigInteger::ONE.clone();
                    let mut real_accumulator = BigFloat::with_val(DEFAULT_REAL_PRECISION, 1);
                    let mut seen_real = false;

                    let mut new_elements = Vec::with_capacity(expr_elements.len());

                    for expr in expr_elements {
                        match expr.kind() {
                            ExprKind::Integer(n) => {
                                int_accumulator.mul_assign(n);
                            }
                            ExprKind::Real(r) => {
                                real_accumulator.mul_assign(r.as_float());
                                seen_real = true;
                            }
                            _ => {
                                new_elements.push(expr.clone());
                            }
                        }
                    }

                    if seen_real {
                        real_accumulator.mul_assign(&int_accumulator);
                        new_elements.push(Expr::from(real_accumulator));
                    } else if int_accumulator != 0 {
                        new_elements.push(Expr::from(int_accumulator));
                    }

                    if new_elements.len() == 0 {
                        EvalResult::Changed(Expr::from(BigInteger::ONE.clone()))
                    } else if new_elements.len() == 1 {
                        EvalResult::Changed(new_elements[0].clone())
                    } else if new_elements != expr_elements {
                        EvalResult::Changed(Expr::from(Normal::new(sym!(Times), new_elements)))
                    } else {
                        EvalResult::Unchanged(expr)
                    }
                },
            },
        )
        .unwrap();

    context
        .set_attributes(
            &sym!(Times),
            Attribute::ReadOnly
                + Attribute::AttributesReadOnly
                + Attribute::Associative
                + Attribute::Commutative,
        )
        .unwrap();
}

/// Registers the `Subtract` builtin symbol.
///
/// - `Attributes[Subtract] = { ReadOnly, AttributesReadOnly }`
/// - `Subtract[lhs_, rhs_] := built-in`
pub(crate) fn register_subtract_builtin(context: &mut Context) {
    context
        .set_value(
            &sym!(Subtract),
            ValueType::DownValue,
            SymbolValue::BuiltIn {
                pattern: parse!("Subtract[lhs_, rhs_]"),
                condition: None,
                built_in: |arguments, _, _| {
                    let lhs = &arguments[&Symbol::new("lhs")];
                    let rhs = &arguments[&Symbol::new("rhs")];

                    EvalResult::Changed(Expr::from(Normal::new(
                        sym!(Plus),
                        vec![
                            lhs.clone(),
                            Expr::from(Normal::new(
                                sym!(Times),
                                vec![Expr::from(BigInteger::NEG_ONE.clone()), rhs.clone()],
                            )),
                        ],
                    )))
                },
            },
        )
        .unwrap();

    context
        .set_attributes(
            &sym!(Subtract),
            Attribute::ReadOnly + Attribute::AttributesReadOnly,
        )
        .unwrap();
}
