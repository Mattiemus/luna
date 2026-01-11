use crate::{
    Atom, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Substitution,
    is_blank_pattern,
};

/// Function variable elimination.
///
/// Matches a pattern `<f_>[...]` against a value `g[...]`.
pub(crate) struct RuleFVE {
    match_equation: MatchEquation,
    exhausted: bool,
}

impl RuleFVE {
    pub fn new(match_equation: MatchEquation) -> Self {
        Self {
            match_equation,
            exhausted: false,
        }
    }
}

impl MatchRule for RuleFVE {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        // Pattern: <f_>[...]
        // Ground:  g[...]

        if is_blank_pattern(&match_equation.pattern.head())
            && match_equation.ground.is_expr()
        {
            Some(RuleFVE::new(match_equation.clone()))
        } else {
            None
        }
    }
}

impl MatchGenerator for RuleFVE {
    fn match_equation(&self) -> MatchEquation {
        self.match_equation.clone()
    }
}

impl Iterator for RuleFVE {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        // Create a substitution of `f_` to `g`.
        let substitution = MatchResult::Substitution(Substitution {
            variable: self.match_equation.pattern.head().clone(),
            ground: self.match_equation.ground.head().clone(),
        });

        // Next steps are to solve for the arguments of the pattern `<f_>[...]`.
        // Create a new match equation with the head of the pattern replaced with the head of the
        // ground.
        //
        // For example `<f_>[a, b, c]` becomes `g[a, b, c]`.
        let new_match_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Atom::expr(
                self.match_equation.ground.head().clone(),
                self.match_equation.pattern.children(),
            ),
            ground: self.match_equation.ground.clone(),
        });

        self.exhausted = true;
        Some(vec![new_match_equation, substitution])
    }
}
