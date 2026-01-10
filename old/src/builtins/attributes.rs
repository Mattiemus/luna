use crate::ast::Expr;
use crate::builtins::LIST;
use crate::symbol::{Attributes, SymbolDef};
use std::sync::LazyLock;

// https://reference.wolfram.com/language/ref/Attributes.html
pub static ATTRIBUTES: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("Attributes")
        .with_attributes(Attributes::PROTECTED | Attributes::HOLD_ALL | Attributes::LISTABLE)
        .with_builtin(|args, env| {
            match args.len() {
                // Attributes[x]
                1 => match args[0] {
                    // Attributes[symbol]
                    Expr::Symbol(symbol_id) => {
                        let attributes = env.symbol_def(symbol_id).attributes;

                        let attribute_exprs = Vec::new();

                        // TODO

                        Ok(Expr::apply_symbol(LIST.symbol_id(), attribute_exprs))
                    }

                    // Attributes[...]
                    _ => {
                        // Unexpected argument type
                        Ok(Expr::apply_symbol(ATTRIBUTES.symbol_id(), args))
                    }
                },

                // Attributes[...]
                _ => {
                    // Unexpected number of arguments
                    Ok(Expr::apply_symbol(ATTRIBUTES.symbol_id(), args))
                }
            }
        })
});

#[cfg(test)]
mod tests {
    use crate::env::Env;
    use crate::fullform::assert_eval;

    // #[test]
    // fn has_attributes() {
    //     let mut env = Env::new();
    //
    //     assert_eval(
    //         &mut env,
    //         "Attributes[Attributes]",
    //         "List[HoldAll, Listable, Protected]",
    //     );
    // }
}
