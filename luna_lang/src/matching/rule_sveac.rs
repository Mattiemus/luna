use crate::matching::function_application::{AFACGenerator, FunctionApplicationGenerator};
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_any_sequence_variable,
};
use crate::matching::subsets::next_subset;

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

    /// Bit positions indicate which subset of the ground's arguments are currently being matched
    /// against.
    subset: u32,

    /// List of values that are not part of the current subset.
    complement: Vec<Expr>,

    /// Generator to produce associative-commutative function applications.
    /// This being `None` indicates we still need to produce an empty sequence.
    afa_generator: Option<Box<AFACGenerator>>,
}

impl RuleSVEAC {
    fn make_next(&self, ordered_sequence: Vec<Expr>) -> MatchResultList {
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
                        subset: 0,
                        complement: Vec::with_capacity(g.len()),
                        afa_generator: if matches_empty {
                            None
                        } else {
                            Some(Box::new(AFACGenerator::new(Normal::new(
                                g.head().clone(),
                                vec![],
                            ))))
                        },
                    });
                }
            }
        }

        None
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
        match &mut self.afa_generator {
            None => {
                self.complement.clear();
                self.complement.extend_from_slice(self.ground.elements());

                self.afa_generator = Some(Box::new(AFACGenerator::new(Normal::new(
                    self.ground.head().clone(),
                    vec![],
                ))));

                Some(self.make_next(Vec::new()))
            }

            Some(afa_generator) => {
                let ordered_sequence = match afa_generator.next() {
                    None => {
                        if let Some(next_subset) =
                            next_subset(self.ground.len() as u32, self.subset)
                        {
                            self.subset = next_subset;

                            let mut subset = Vec::with_capacity(self.subset.count_ones() as usize);
                            self.complement.clear();

                            for (k, c) in self.ground.elements().iter().enumerate() {
                                if ((1 << k) as u32 & self.subset) != 0 {
                                    subset.push(c.clone());
                                } else {
                                    self.complement.push(c.clone());
                                }
                            }

                            let mut new_afa_generator =
                                AFACGenerator::new(Normal::new(self.ground.head().clone(), subset));

                            let next_result = new_afa_generator.next().unwrap();
                            self.afa_generator = Some(Box::new(new_afa_generator));

                            next_result
                        } else {
                            return None;
                        }
                    }

                    Some(next_result) => next_result,
                };

                Some(self.make_next(ordered_sequence))
            }
        }
    }
}
