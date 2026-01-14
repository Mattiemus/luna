use crate::Normal;
use crate::Symbol;
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Substitution,
    parse_individual_variable,
};

/// Function variable elimination.
///
/// Matches a pattern `f_[___]` against a value `g[___]`.
pub(crate) struct RuleFVE {
    pattern: Normal,
    ground: Normal,
    variable: Option<Symbol>,
    exhausted: bool,
}

impl MatchRule for RuleFVE {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if let (Some(p), Some(g)) = (
            match_equation.pattern.try_normal(),
            match_equation.ground.try_normal(),
        ) {
            if let Some((variable, _)) = parse_individual_variable(p.head()) {
                // TODO: Evaluate constraints for `Blank[h]` and `Pattern[_, Blank[h]]`.

                return Some(Self {
                    pattern: p.clone(),
                    ground: g.clone(),
                    variable: variable.cloned(),
                    exhausted: false,
                });
            }
        }

        None
    }
}

impl MatchGenerator for RuleFVE {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleFVE {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        self.exhausted = true;

        // Next steps are to solve for the arguments of the pattern `f_[...]`.
        // Create a new match equation with the head of the pattern replaced with the head of the
        // ground.
        //
        // For example `f_[a, b, c]` becomes `g[a, b, c]`.
        let new_match_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Expr::from(Normal::new(
                self.ground.head().clone(),
                self.pattern.elements(),
            )),
            ground: Expr::from(self.ground.clone()),
        });

        if let Some(variable) = &self.variable {
            let substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                ground: Expr::from(self.ground.head().clone()),
            });

            return Some(vec![new_match_equation, substitution]);
        }

        Some(vec![new_match_equation])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MatchEquation, MatchRule, parse};
    use test_case::test_case;

    #[test_case("\"abc\"[a, b, c]", "\"abc\"[___]", "\"abc\"" ; "string")]
    #[test_case("123[a, b, c]", "123[___]", "123" ; "integer")]
    #[test_case("1.25[a, b, c]", "1.25[___]", "1.25" ; "real")]
    #[test_case("abc[a, b, c]", "abc[___]", "abc" ; "symbol")]
    #[test_case("f[a, b, c][x, y, z]", "f[a, b, c][___]", "f[a, b, c]" ; "expression")]
    fn matches_head_named_variable(ground: &str, next_pattern: &str, substitution: &str) {
        let mut rule = RuleFVE::try_rule(&MatchEquation {
            pattern: parse("x_[___]").unwrap(),
            ground: parse(ground).unwrap(),
        })
        .unwrap();

        assert_eq!(
            rule.next(),
            Some(vec![
                MatchResult::MatchEquation(MatchEquation {
                    pattern: parse(next_pattern).unwrap(),
                    ground: parse(ground).unwrap(),
                }),
                MatchResult::Substitution(Substitution {
                    variable: Symbol::new("x"),
                    ground: parse(substitution).unwrap(),
                })
            ])
        );

        assert_eq!(rule.next(), None);
    }

    #[test_case("\"abc\"[a, b, c]", "\"abc\"[___]" ; "string")]
    #[test_case("123[a, b, c]", "123[___]" ; "integer")]
    #[test_case("1.25[a, b, c]", "1.25[___]" ; "real")]
    #[test_case("abc[a, b, c]", "abc[___]" ; "symbol")]
    #[test_case("f[a, b, c][x, y, z]", "f[a, b, c][___]" ; "expression")]
    fn matches_head_unnamed_variable(ground: &str, next_pattern: &str) {
        let mut rule = RuleFVE::try_rule(&MatchEquation {
            pattern: parse("_[___]").unwrap(),
            ground: parse(ground).unwrap(),
        })
        .unwrap();

        assert_eq!(
            rule.next(),
            Some(vec![MatchResult::MatchEquation(MatchEquation {
                pattern: parse(next_pattern).unwrap(),
                ground: parse(ground).unwrap(),
            })])
        );
        assert_eq!(rule.next(), None);
    }

    #[test_case("abc[a, b, c]", "abc[a, b, c]" ; "pattern is not variable")]
    fn does_not_match(pattern: &str, ground: &str) {
        let rule = RuleFVE::try_rule(&MatchEquation {
            pattern: parse(pattern).unwrap(),
            ground: parse(ground).unwrap(),
        });

        assert!(rule.is_none());
    }
}
