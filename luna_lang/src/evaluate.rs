use crate::{Atom, Context};

pub fn evaluate_once(expression: &Atom, context: &mut Context) -> Atom {
    match expression {
        Atom::Symbol(symbol) => {
            todo!()

            // let unevaluated = find_matching_definition(
            //     expression.clone(),
            //     symbol.clone(),
            //     ContextValueStore::OwnValues,
            //     context
            // );

            // if unevaluated.is_some() {
            //     unevaluated.apply(expression, context)
            // } else {
            //     // No own-values. Return original expression.
            //     expression
            // }
        }

        Atom::SExpression(children) => {
            todo!()
        }

        // String, Integer, and Real remain unchanged
        _ => expression.clone(),
    }
}
