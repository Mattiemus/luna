use crate::ast::Expr;
use crate::symbol::{Attributes, SymbolDef};
use num_bigint::BigInt;
use std::sync::LazyLock;

// https://reference.wolfram.com/language/ref/Plus.html
pub static PLUS: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("Plus")
        .with_attributes(
            Attributes::PROTECTED
                | Attributes::FLAT
                | Attributes::ORDERLESS
                | Attributes::ONE_IDENTITY
                | Attributes::LISTABLE
                | Attributes::NUMERIC_FUNCTION,
        )
        .with_builtin(|args, env| {
            let mut terms: Vec<Expr> = Vec::new();
            let mut int_sum = BigInt::ZERO;

            for arg in args {
                match arg {
                    Expr::Integer(i) => {
                        int_sum += i;
                    }
                    other => terms.push(other),
                }
            }

            match terms.len() {
                // If no terms remaining forward the accumulator.
                0 => Ok(Expr::integer(int_sum)),

                // Otherwise forward the remaining terms, and prepending the accumulator
                // if there were any integer values.
                _ => {
                    if int_sum != BigInt::ZERO {
                        terms.insert(0, Expr::integer(int_sum));
                    }

                    Ok(Expr::apply_symbol(PLUS.symbol_id(), terms))
                }
            }
        })
});

#[cfg(test)]
mod tests {
    use crate::env::Env;
    use crate::fullform::assert_eval;

    #[test]
    fn basics() {
        let mut env = Env::new();

        assert_eval(&mut env, "Plus[]", "0");
        assert_eval(&mut env, "Plus[1]", "1");
        assert_eval(&mut env, "Plus[1, 2]", "3");
        assert_eval(&mut env, "Plus[1, 2, 3]", "6");

        assert_eval(&mut env, "Plus[x]", "x");
        assert_eval(&mut env, "Plus[x, 0]", "x");
        assert_eval(&mut env, "Plus[0, x]", "x");
        assert_eval(&mut env, "Plus[0, x, 0]", "x");
    }

    #[test]
    fn flattening() {
        let mut env = Env::new();

        assert_eval(&mut env, "Plus[1, Plus[2, 3]]", "6");
        assert_eval(&mut env, "Plus[1, Plus[2, Plus[3]]]", "6");
        assert_eval(&mut env, "Plus[a, Plus[b, c]]", "Plus[a, b, c]");
    }

    #[test]
    fn orderless() {
        let mut env = Env::new();

        assert_eval(&mut env, "Plus[b, a]", "Plus[a, b]");
        assert_eval(&mut env, "Plus[x, 2, y, 1]", "Plus[3, x, y]");
    }

    #[test]
    fn listable() {
        let mut env = Env::new();

        // Multiple lists
        assert_eval(&mut env, "Plus[List[1, 2], List[3, 4]]", "List[4, 6]");
        assert_eval(
            &mut env,
            "Plus[List[1, 2, 3], List[4, 5, 6]]",
            "List[5, 7, 9]",
        );

        // Nestable lists
        assert_eval(&mut env, "Plus[List[1, 2, 3], 10]", "List[11, 12, 13]");
        assert_eval(
            &mut env,
            "Plus[List[x, y], 1]",
            "List[Plus[1, x], Plus[1, y]]",
        );
        assert_eval(&mut env, "Plus[List[1, 2], List[3, 4], 10]", "List[14, 16]");

        // Flat + Orderless
        assert_eval(&mut env, "Plus[List[1, 2], Plus[3, 4]]", "List[8, 9]");

        // Listable + OneIdentity
        assert_eval(&mut env, "Plus[List[x]]", "List[x]");

        // Failure case (unbalanced list)
        assert_eval(
            &mut env,
            "Plus[List[1, 2], List[3, 4, 5]]",
            "Plus[List[1, 2], List[3, 4, 5]]",
        );
    }
}
