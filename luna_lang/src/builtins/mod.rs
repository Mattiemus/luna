use crate::{Attribute, RewriteRule, parse};
use crate::{Context, Expr, SolutionSet};
use crate::{ExprKind, Symbol, extract_condition};
use crate::{Normal, ValueType};

pub type BuiltinFn = fn(SolutionSet, Expr, &Context) -> Expr;
pub type BuiltinFnMut = fn(SolutionSet, Expr, &mut Context) -> Expr;

/// Registers all builtins.
pub(crate) fn register_builtins(context: &mut Context) {
    register_set_builtin(context);
    register_head_builtin(context);
}

/// Registers the `Set` builtin symbol.
///
/// - `Attributes[Set] = { ReadOnly, AttributesReadOnly, HoldFirst, SequenceHold }`
/// - `Set[lhs_, rhs_] := built-in`
pub(crate) fn register_set_builtin(context: &mut Context) {
    context
        .set_value(
            &Symbol::new("Set"),
            ValueType::DownValue,
            RewriteRule::BuiltInMut {
                pattern: parse!("Set[lhs_, rhs_]"),
                condition: None,
                built_in: |arguments, _, context| {
                    let pattern = &arguments[&Symbol::new("lhs")];
                    let ground = &arguments[&Symbol::new("rhs")];

                    let (ground, condition) = extract_condition(ground.clone());

                    let rewrite_rule = RewriteRule::Definitions {
                        pattern: pattern.clone(),
                        condition,
                        ground: ground.clone(),
                    };

                    let name = pattern.name().unwrap();

                    match pattern.kind() {
                        ExprKind::Symbol(_) => {
                            context
                                .set_value(name, ValueType::OwnValue, rewrite_rule)
                                .unwrap();
                        }
                        ExprKind::Normal(_) => {
                            context
                                .set_value(name, ValueType::DownValue, rewrite_rule)
                                .unwrap();
                        }
                        _ => todo!(),
                    }

                    Expr::from(Normal::new(Symbol::new("Hold"), vec![ground]))
                },
            },
        )
        .unwrap();

    context
        .set_attributes(
            &Symbol::new("Set"),
            Attribute::ReadOnly
                + Attribute::AttributesReadOnly
                + Attribute::HoldFirst
                + Attribute::SequenceHold,
        )
        .unwrap();
}

/// Registers the `Head` builtin symbol.
///
/// - `Attributes[Head] = { ReadOnly, AttributesReadOnly }`
/// - `Head[expr_] := built-in`
/// - `Head[expr_, h_] := built-in`
pub(crate) fn register_head_builtin(context: &mut Context) {
    context
        .set_value(
            &Symbol::new("Head"),
            ValueType::DownValue,
            RewriteRule::BuiltIn {
                pattern: parse!("Head[expr_]"),
                condition: None,
                built_in: |arguments, _, _| {
                    let expr = &arguments[&Symbol::new("expr")];

                    expr.head().clone()
                },
            },
        )
        .unwrap();

    context
        .set_value(
            &Symbol::new("Head"),
            ValueType::DownValue,
            RewriteRule::BuiltIn {
                pattern: parse!("Head[expr_, h_]"),
                condition: None,
                built_in: |arguments, _, _| {
                    let expr = &arguments[&Symbol::new("expr")];
                    let h = &arguments[&Symbol::new("h")];

                    Expr::from(Normal::new(h.clone(), vec![expr.head().clone()]))
                },
            },
        )
        .unwrap();

    context
        .set_attributes(
            &Symbol::new("Head"),
            Attribute::ReadOnly + Attribute::AttributesReadOnly,
        )
        .unwrap();
}
