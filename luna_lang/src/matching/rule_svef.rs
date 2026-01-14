use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_any_sequence_variable,
};

/// Sequence variable elimination under a free head.
///
/// Matches a pattern `f[x__, ...]` or `f[x___, ...]` against a value `g[y, ...]` where `f` is a
/// free function.
///
/// Assumptions:
/// - `f` is a free function.
/// - `f` and `g` are equal.
pub(crate) struct RuleSVEF {
    pattern: Normal,
    ground: Normal,
    variable: Option<Symbol>,

    /// Have we produced the empty sequence as the first result yet?
    empty_produced: bool,

    /// A `Sequence`, holds the terms of the ground that we have attempted to match against so far.
    ground_sequence: Vec<Expr>,
}

impl RuleSVEF {
    fn make_next(&self) -> MatchResultList {
        let new_match_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Expr::from(Normal::new(
                self.pattern.head().clone(),
                &self.pattern.elements()[1..],
            )),
            ground: Expr::from(Normal::new(
                self.ground.head().clone(),
                &self.ground.elements()[self.ground_sequence.len()..],
            )),
        });

        if let Some(variable) = &self.variable {
            let substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                ground: Expr::from(Normal::new(
                    Symbol::new("Sequence"),
                    self.ground_sequence.clone(),
                )),
            });

            return vec![new_match_equation, substitution];
        }

        vec![new_match_equation]
    }
}

impl MatchRule for RuleSVEF {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        if let (Some(p), Some(g)) = (
            match_equation.pattern.try_normal(),
            match_equation.ground.try_normal(),
        ) {
            if let Some(p0) = p.part(0) {
                if let Some((matches_empty, variable, _)) = parse_any_sequence_variable(p0) {
                    // TODO: Evaluate constraints for `BlankSequence[h]` and `Pattern[_, BlankSequence[h]]`.

                    return Some(Self {
                        pattern: p.clone(),
                        ground: g.clone(),
                        variable: variable.cloned(),
                        empty_produced: !matches_empty,
                        ground_sequence: vec![],
                    });
                }
            }
        }

        None
    }
}

impl MatchGenerator for RuleSVEF {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleSVEF {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        // If we haven't produced the empty sequence, do that.
        if !self.empty_produced {
            self.empty_produced = true;
            return Some(self.make_next());
        }

        // Take the next term from the ground function.
        if let Some(next_part) = self.ground.part(self.ground_sequence.len()) {
            self.ground_sequence.push(next_part.clone());
        } else {
            return None;
        }

        // Construct the result.
        Some(self.make_next())
    }
}
