use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_any_sequence_variable,
};

/// Sequence variable elimination under a free head.
///
/// Matches a pattern `f[x__, ...]` or `f[x___, ...]` against a value `g[...]` where `f` is a
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

    /// Holds the terms of the ground that we have attempted to match against so far.
    ground_sequence: Vec<Expr>,
}

impl RuleSVEF {
    pub(crate) fn new(
        pattern: Normal,
        ground: Normal,
        variable: Option<Symbol>,
        matches_empty: bool,
    ) -> Self {
        Self {
            pattern,
            ground,
            variable,
            empty_produced: !matches_empty,
            ground_sequence: vec![],
        }
    }
}

impl RuleSVEF {
    fn make_next(&self) -> MatchResultList {
        // Attempt to continue to match `f[...]` against `g[...]`.
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

        // Create the substitution so long as the pattern was named.
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
        let p = match_equation.pattern.try_normal()?;
        let g = match_equation.ground.try_normal()?;

        let p_elem0 = p.element(0)?;
        let (matches_empty, variable, _) = parse_any_sequence_variable(p_elem0)?;

        // TODO: Evaluate constraints for `BlankSequence[h]` and `Pattern[_, BlankSequence[h]]`.

        Some(Self::new(
            p.clone(),
            g.clone(),
            variable.cloned(),
            matches_empty,
        ))
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
        let next_element = self.ground.element(self.ground_sequence.len())?;
        self.ground_sequence.push(next_element.clone());

        // Construct the result.
        Some(self.make_next())
    }
}
