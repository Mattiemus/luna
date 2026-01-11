use crate::{Atom, Context, SExpression, SolutionSet, SymbolValue};
use crate::{Attribute, parse};

pub type BuiltinFn = fn(Atom, SolutionSet, &Context) -> Atom;
pub type BuiltinFnMut = fn(Atom, SolutionSet, &mut Context) -> Atom;

/// Registers all builtins.
pub(crate) fn register_builtins(context: &mut Context) {
    // TODO: This should register built ins once we have a simple parser ready.
    // register_head_builtin(context);
}

/// Registers the `Head` builtin symbol.
///
/// - `Attributes[Head] = { ReadOnly, AttributesReadOnly }`
/// - `Head[expr_] := built-in[expr]`
/// - `Head[expr_, h_] := built-in[expr, h]`
pub(crate) fn register_head_builtin(context: &mut Context) {
    context
        .set_down_value(
            "Head",
            SymbolValue::BuiltIn {
                pattern: parse!("Head[expr_]"),
                condition: None,
                built_in: |_, arguments, _| {
                    let expr = &arguments[&parse!("expr_")];

                    expr.head()
                },
            },
        )
        .unwrap();

    context
        .set_down_value(
            "Head",
            SymbolValue::BuiltIn {
                pattern: parse!("Head[expr_, h_]"),
                condition: None,
                built_in: |_, arguments, _| {
                    let expr = &arguments[&parse!("expr_")];
                    let h = &arguments[&parse!("h_")];

                    SExpression::apply1(h.clone(), expr.head()).into()
                },
            },
        )
        .unwrap();

    context
        .set_attributes("Head", Attribute::ReadOnly + Attribute::AttributesReadOnly)
        .unwrap();
}
