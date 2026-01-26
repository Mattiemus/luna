use crate::matching::rule_dnc::RuleDNC;
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_individual_variable,
};

/// Function variable elimination under an associative head.
///
/// Matches a pattern `f[f_[...], ...]` against a value `g[...]` where `f` is an associative
/// function.
///
/// Assumptions:
/// - `f` is an associative function.
/// - `f` and `g` are equal.
pub struct RuleFVEA {
    pattern: Normal,
    pattern_first: Normal,
    ground: Normal,
    variable: Option<Symbol>,
    rule_dnc: RuleDNC,
    exhausted: bool,
}

impl RuleFVEA {
    pub(crate) fn new(
        pattern: Normal,
        pattern_first: Normal,
        ground: Normal,
        variable: Option<Symbol>,
    ) -> Self {
        let rule_deca = RuleDNC::new(pattern.clone(), ground.clone());

        Self {
            pattern,
            pattern_first,
            ground,
            variable,
            rule_dnc: rule_deca,
            exhausted: false,
        }
    }
}

impl MatchRule for RuleFVEA {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        let p = match_equation.pattern.try_normal()?;
        let g = match_equation.ground.try_normal()?;

        let (p_elem0, _) = (p.element(0)?, g.element(0)?);
        let p_elem0_normal = p_elem0.try_normal()?;
        let (variable, _) = parse_individual_variable(p_elem0_normal.head())?;

        // TODO: Evaluate constraints for `Blank[h]` and `Pattern[_, Blank[h]]`.

        Some(Self::new(
            p.clone(),
            p_elem0_normal.clone(),
            g.clone(),
            variable.cloned(),
        ))
    }
}

impl MatchGenerator for RuleFVEA {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleFVEA {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(result) = self.rule_dnc.next() {
            return Some(result);
        }

        if self.exhausted {
            return None;
        }

        self.exhausted = true;

        let new_match_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Expr::from(Normal::new(
                self.ground.head().clone(),
                self.pattern_first
                    .elements()
                    .iter()
                    .chain(self.pattern.elements()[1..].iter())
                    .cloned()
                    .collect::<Vec<_>>(),
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
