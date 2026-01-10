use crate::ast::Expr;
use crate::symbol::{Attributes, SymbolDef};
use std::sync::LazyLock;

// https://reference.wolfram.com/language/ref/Head.html
pub static HEAD: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("Head")
        .with_attributes(Attributes::PROTECTED)
        .with_builtin(|args, _env| {
            match args.len() {
                // Head[expr] (gets the head of `expr`)
                1 => Ok(args[0].head()),

                // Head[expr, h] (gets the head of `expr` and wraps the result with `h`)
                2 => Ok(Expr::apply(args[1].clone(), vec![args[0].head()])),

                // Head[...]
                _ => {
                    // Unexpected number of arguments
                    Ok(Expr::apply_symbol(HEAD.symbol_id(), args))
                }
            }
        })
});

#[cfg(test)]
mod tests {
    use crate::env::Env;
    use crate::fullform::assert_eval;

    #[test]
    fn atoms() {
        let mut env = Env::new();

        assert_eval(&mut env, "Head[x]", "Symbol");
        assert_eval(&mut env, "Head[123]", "Integer");

        assert_eval(&mut env, "Head[x, f]", "f[Symbol]");
        assert_eval(&mut env, "Head[123, f]", "f[Integer]");
    }

    #[test]
    fn application() {
        let mut env = Env::new();

        assert_eval(&mut env, "Head[f[x]]", "f");
        assert_eval(&mut env, "Head[f[x][y]]", "f[x]");
        assert_eval(&mut env, "Head[f[x][y][z]]", "f[x][y]");

        assert_eval(&mut env, "Head[f[x], g]", "g[f]");
        assert_eval(&mut env, "Head[f[x][y], g]", "g[f[x]]");
        assert_eval(&mut env, "Head[f[x][y][z], g]", "g[f[x][y]]");
    }

    #[test]
    fn runs_after_normalization() {
        let mut env = Env::new();

        assert_eval(&mut env, "Head[Plus[1, Plus[2, 3]]]", "Integer");
        assert_eval(&mut env, "Head[Plus[x]]", "Symbol");

        assert_eval(&mut env, "Head[Plus[1, Plus[2, 3]], f]", "f[Integer]");
        assert_eval(&mut env, "Head[Plus[x], f]", "f[Symbol]");
    }
}
