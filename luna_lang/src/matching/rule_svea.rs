use crate::matching::function_application::{AFAGenerator, FunctionApplicationGenerator};
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_any_sequence_variable, sym,
};

/// Sequence variable elimination under an associative head.
///
/// Matches a pattern `f[x__, ...]` or `f[x___, ...]` against a value `g[...]` where `f` is an
/// associative function.
///
/// Assumptions:
/// - `f` is an associative function.
/// - `f` and `g` are equal.
pub(crate) struct RuleSVEA {
    pattern: Normal,
    ground: Normal,
    variable: Option<Symbol>,

    /// Holds the terms of the ground that we have attempted to match against so far.
    ground_sequence: Vec<Expr>,

    /// Generator to produce associative function applications.
    /// This being `None` indicates we still need to produce an empty sequence.
    afa_generator: Option<Box<AFAGenerator>>,
}

impl RuleSVEA {
    pub(crate) fn new(
        pattern: Normal,
        ground: Normal,
        variable: Option<Symbol>,
        matches_empty: bool,
    ) -> Self {
        let afa_generator = if matches_empty {
            None
        } else {
            Some(Box::new(AFAGenerator::new(Normal::new(
                ground.head().clone(),
                vec![],
            ))))
        };

        Self {
            pattern,
            ground,
            variable,
            ground_sequence: Vec::new(),
            afa_generator,
        }
    }

    fn make_next(&self, ordered_sequence: Vec<Expr>) -> MatchResultList {
        // Attempt to continue to match `f[...]` against `g[...]`.
        let result_equation = MatchResult::MatchEquation(MatchEquation {
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
            let result_substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                ground: Expr::from(Normal::new(sym!(Sequence), ordered_sequence)),
            });

            return vec![result_equation, result_substitution];
        }

        vec![result_equation]
    }
}

impl MatchRule for RuleSVEA {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        let p = match_equation.pattern.try_normal()?;
        let g = match_equation.ground.try_normal()?;

        let p_elem0 = p.element(0)?;
        let (matches_empty, variable, _) = parse_any_sequence_variable(p_elem0)?;

        // TODO: Evaluate constraints for `BlankSequence[h]` and `Pattern[_, BlankSequence[h]]`.

        // If we are the final part of the pattern then it only makes sense to start looking for
        // matches starting with the contents of the ground.
        // This optimization prevents us producing a large number of unsolvable match equations.
        if p.len() == 1 && !g.is_empty() {
            // TODO: This optimisation can apply here!
        }

        Some(Self::new(
            p.clone(),
            g.clone(),
            variable.cloned(),
            matches_empty,
        ))
    }
}

impl MatchGenerator for RuleSVEA {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleSVEA {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.afa_generator {
            // Current generator being `None` is the signal we need to produce an empty sequence.
            None => {
                self.afa_generator = Some(Box::new(AFAGenerator::new(Normal::new(
                    self.ground.head().clone(),
                    vec![],
                ))));

                Some(self.make_next(Vec::new()))
            }

            // Otherwise generate the next result.
            Some(afa_generator) => {
                // Determine the next sequence.
                let ordered_sequence = match afa_generator.next() {
                    // There is no next valid result from the AFA generator.
                    // Copy the next element from the `ground` expression into the active ground
                    // sequence, and use that as the basis for a new AFA generator.
                    None => {
                        // Attempt to extend the ground sequence
                        let next_element = self.ground.element(self.ground_sequence.len())?;
                        self.ground_sequence.push(next_element.clone());

                        // Use the ground sequence to build a new AFA generator.
                        let mut new_afa_generator = AFAGenerator::new(Normal::new(
                            self.ground.head().clone(),
                            self.ground_sequence.clone(),
                        ));

                        // Get the next result to be returned and store the new AFA generator.
                        let next_result = new_afa_generator.next().unwrap();
                        self.afa_generator = Some(Box::new(new_afa_generator));

                        next_result
                    }

                    // Current AFA generator is still producing values.
                    Some(next_result) => next_result,
                };

                // Transform the sequence into a result.
                Some(self.make_next(ordered_sequence))
            }
        }
    }
}
