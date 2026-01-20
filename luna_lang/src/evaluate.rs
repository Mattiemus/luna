use crate::{Context, Expr};

pub fn evaluate(expr: &Expr, _context: &mut Context) -> Expr {
    match expr.head() {
        //     Expr::Symbol(_symbol) => {
        //         todo!()
        //
        //         // let unevaluated = find_matching_definition(
        //         //     expression.clone(),
        //         //     symbol.clone(),
        //         //     ContextValueStore::OwnValues,
        //         //     context
        //         // );
        //
        //         // if unevaluated.is_some() {
        //         //     unevaluated.apply(expression, context)
        //         // } else {
        //         //     // No own-values. Return original expression.
        //         //     expression
        //         // }
        //     }
        //
        //     Expr::Normal(_expr) => {
        //         todo!()
        //     }
        //
        // String, Integer, and Real remain unchanged
        _ => expr.clone(),
    }
}
