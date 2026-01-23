use crate::matching::function_application::{AFACGenerator, FunctionApplicationGenerator};
use crate::matching::subsets::Subset;
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_any_sequence_variable,
};

/// Sequence variable elimination under an associative-commutative head.
///
/// Matches a pattern `f[x__, ...]` or `f[x___, ...]` against a value `g[...]` where `f` is an
/// associative function.
///
/// Assumptions:
/// - `f` is an associative-commutative function.
/// - `f` and `g` are equal.
pub(crate) struct RuleSVEAC {
    pattern: Normal,
    ground: Normal,
    variable: Option<Symbol>,

    /// Current subset of the grounds arguments which are being matched against.
    subset: Subset,

    /// List of values that are not part of the current subset.
    complement: Vec<Expr>,

    /// Generator to produce associative-commutative function applications.
    /// This being `None` indicates we still need to produce an empty sequence.
    afac_generator: Option<Box<AFACGenerator>>,
}

impl RuleSVEAC {
    pub(crate) fn new(
        pattern: Normal,
        ground: Normal,
        variable: Option<Symbol>,
        matches_empty: bool,
    ) -> Self {
        let subset = Subset::empty(ground.len());

        let afac_generator = if matches_empty {
            None
        } else {
            Some(Box::new(AFACGenerator::new(Normal::new(
                ground.head().clone(),
                vec![],
            ))))
        };

        Self {
            pattern,
            ground,
            variable,
            subset,
            complement: vec![],
            afac_generator,
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
                self.complement.clone(),
            )),
        });

        // Create the substitution so long as the pattern was named.
        if let Some(variable) = &self.variable {
            let result_substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                ground: Expr::from(Normal::new(Symbol::new("Sequence"), ordered_sequence)),
            });

            return vec![result_equation, result_substitution];
        }

        vec![result_equation]
    }
}

impl MatchRule for RuleSVEAC {
    fn try_rule(match_equation: &MatchEquation) -> Option<Self> {
        let p = match_equation.pattern.try_normal()?;
        let g = match_equation.ground.try_normal()?;

        let p0 = p.part(0)?;
        let (matches_empty, variable, _) = parse_any_sequence_variable(p0)?;

        // TODO: Evaluate constraints for `BlankSequence[h]` and `Pattern[_, BlankSequence[h]]`.

        Some(Self::new(
            p.clone(),
            g.clone(),
            variable.cloned(),
            matches_empty,
        ))
    }
}

impl MatchGenerator for RuleSVEAC {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleSVEAC {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.afac_generator {
            // Current generator being `None` is the signal we need to produce an empty sequence.
            None => {
                self.complement = self.ground.elements().to_vec();

                self.afac_generator = Some(Box::new(AFACGenerator::new(Normal::new(
                    self.ground.head().clone(),
                    vec![],
                ))));

                Some(self.make_next(Vec::new()))
            }

            // Otherwise generate the next result.
            Some(afac_generator) => {
                // Determine the next sequence.
                let ordered_sequence = match afac_generator.next() {
                    // There is no next valid result from the AFAC generator.
                    // Determine the next subset of elements from `g` to use and construct a new
                    // AFAC generator.
                    None => {
                        // Determine the next subset.
                        self.subset = self.subset.next()?;

                        // Extract the subset and complement from the current subset
                        let (subset, complement) = self.subset.extract(self.ground.elements());
                        self.complement = complement;

                        // Use the subset to build a new AFAC generator.
                        let mut new_afac_generator =
                            AFACGenerator::new(Normal::new(self.ground.head().clone(), subset));

                        // Get the next result to be returned and store the new AFAC generator.
                        let next_result = new_afac_generator.next()?;
                        self.afac_generator = Some(Box::new(new_afac_generator));

                        next_result
                    }

                    // Current AFAC generator is still producing values.
                    Some(next_result) => next_result,
                };

                // Transform the sequence into a result.
                Some(self.make_next(ordered_sequence))
            }
        }
    }
}
