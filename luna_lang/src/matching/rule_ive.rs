use crate::Symbol;
use crate::{
    MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Substitution,
    parse_individual_variable,
};

/// Individual variable elimination.
///
/// Matches a pattern `x_` against any value.
pub(crate) struct RuleIVE {
    match_equation: MatchEquation,
    variable: Option<Symbol>,
    exhausted: bool,
}

impl MatchRule for RuleIVE {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if let Some((variable, _)) = parse_individual_variable(&match_equation.pattern) {
            // TODO: Evaluate constraints for `Blank[h]` and `Pattern[_, Blank[h]]`.

            return Some(Self {
                match_equation: match_equation.clone(),
                variable: variable.cloned(),
                exhausted: false,
            });
        }

        None
    }
}

impl MatchGenerator for RuleIVE {
    fn match_equation(&self) -> MatchEquation {
        self.match_equation.clone()
    }
}

impl Iterator for RuleIVE {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        self.exhausted = true;

        if let Some(variable) = &self.variable {
            let substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                ground: self.match_equation.ground.clone(),
            });

            return Some(vec![substitution]);
        }

        Some(vec![])
    }
}
