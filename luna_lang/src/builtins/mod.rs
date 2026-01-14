use crate::Normal;
use crate::Symbol;
use crate::{Attribute, SymbolValue, parse};
use crate::{Context, Expr, SolutionSet};

pub type BuiltinFn = fn(Expr, SolutionSet, &Context) -> Expr;
pub type BuiltinFnMut = fn(Expr, SolutionSet, &mut Context) -> Expr;

/// Registers all builtins.
pub(crate) fn register_builtins(context: &mut Context) {
    register_head_builtin(context);
}

/// Registers the `Head` builtin symbol.
///
/// - `Attributes[Head] = { ReadOnly, AttributesReadOnly }`
/// - `Head[expr_] := built-in[expr]`
/// - `Head[expr_, h_] := built-in[expr, h]`
pub(crate) fn register_head_builtin(context: &mut Context) {
    context
        .set_down_value(
            &Symbol::new("Head"),
            SymbolValue::BuiltIn {
                pattern: parse!("Head[expr_]"),
                condition: None,
                built_in: |_, arguments, _| {
                    let expr = &arguments[&Symbol::new("expr")];

                    expr.head().clone()
                },
            },
        )
        .unwrap();

    context
        .set_down_value(
            &Symbol::new("Head"),
            SymbolValue::BuiltIn {
                pattern: parse!("Head[expr_, h_]"),
                condition: None,
                built_in: |_, arguments, _| {
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
