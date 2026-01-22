use crate::Normal;
use crate::Symbol;
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Substitution,
    parse_individual_variable,
};

/// Function variable elimination.
///
/// Matches a pattern `f_[...]` against a value `g[...]`.
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

        // Make the substitution of `f_` to `g`, if the pattern was named.
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
