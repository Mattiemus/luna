use crate::{MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Substitution};
use crate::{Symbol, parse_any_sequence_variable, try_sequence};

/// Sequence variable elimination.
///
/// Matches a pattern `x__` and `x___` against any sequence values.
pub(crate) struct RuleSVE {
    match_equation: MatchEquation,
    variable: Option<Symbol>,
    exhausted: bool,
}

impl MatchRule for RuleSVE {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if let Some(gelements) = try_sequence(&match_equation.ground) {
            if let Some((matches_empty, variable, _)) =
                parse_any_sequence_variable(&match_equation.pattern)
            {
                if !matches_empty && gelements.is_empty() {
                    return None;
                }

                // TODO: Evaluate constraints for `BlankSequence[h]` and `Pattern[_, BlankSequence[h]]`.

                return Some(Self {
                    match_equation: match_equation.clone(),
                    variable: variable.cloned(),
                    exhausted: false,
                });
            }
        }

        None
    }
}

impl MatchGenerator for RuleSVE {
    fn match_equation(&self) -> MatchEquation {
        self.match_equation.clone()
    }
}

impl Iterator for RuleSVE {
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
