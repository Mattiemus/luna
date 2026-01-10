// https://reference.wolfram.com/language/guide/Assignments.html

use crate::ast::{Expr, collect_symbols};
use crate::pattern::Rule;
use crate::symbol::{Attributes, SymbolDef};
use std::sync::LazyLock;

// https://reference.wolfram.com/language/ref/Set.html
pub static SET: LazyLock<SymbolDef> = LazyLock::new(|| {
    SymbolDef::new("Set")
        .with_attributes(Attributes::PROTECTED | Attributes::HOLD_FIRST)
        .with_builtin(|args, env| {
            match args.len() {
                // Set[lhs, rhs]
                2 => {
                    let lhs = &args[0];
                    let rhs = &args[1];

                    match lhs {
                        // Set[x, value]
                        Expr::Symbol(symbol_id) => {
                            let def = env.symbol_def_mut(*symbol_id);

                            if def.attributes.contains(Attributes::PROTECTED) {
                                // Symbol is protected
                                return Ok(Expr::apply_symbol(SET.symbol_id(), args));
                            }

                            def.value = Some(rhs.clone());

                            Ok(rhs.clone())
                        }

                        // Set[f[pattern...], rhs]
                        Expr::Apply(_, _) => {
                            let rule = Rule {
                                pattern: lhs.clone(),
                                replacement: rhs.clone(),
                            };

                            // Try DownValue first
                            if let Some(head_sym) = lhs.head_symbol() {
                                env.symbol_def_mut(head_sym).downvalues.push(rule.clone());
                            }

                            // Also attach UpValues
                            let mut syms = Vec::new();
                            collect_symbols(lhs, &mut syms);

                            for sym in syms {
                                env.symbol_def_mut(sym).upvalues.push(rule.clone());
                            }

                            Ok(rhs.clone())
                        }

                        // Unknown
                        _ => Ok(Expr::apply_symbol(SET.symbol_id(), args)),
                    }
                }

                // Set[...]
                _ => {
                    // Unexpected number of arguments
                    Ok(Expr::apply_symbol(SET.symbol_id(), args))
                }
            }
        })
});
