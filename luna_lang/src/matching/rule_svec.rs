use crate::matching::permutations::PermutationGenerator32;
use crate::matching::subsets::Subset;
use crate::{
    Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, MatchRule, Normal,
    Substitution, Symbol, parse_any_sequence_variable, sym,
};

/// Sequence variable elimination under a commutative head.
///
/// Matches a pattern `f[x__, ...]` or `f[x___, ...]` against a value `g[...]` where `f` is a
/// commutative function.
///
/// Assumptions:
/// - `f` is a commutative function.
/// - `f` and `g` are equal.
pub(crate) struct RuleSVEC {
    pattern: Normal,
    ground: Normal,
    variable: Option<Symbol>,

    /// Have we produced the empty sequence as the first result yet?
    empty_produced: bool,

    /// Current subset of the grounds arguments which are being matched against.
    subset: Subset,

    /// Generator for all permutations of the current subset of elements.
    permutations: PermutationGenerator32,
}

impl RuleSVEC {
    pub(crate) fn new(
        pattern: Normal,
        ground: Normal,
        variable: Option<Symbol>,
        matches_empty: bool,
    ) -> Self {
        let subset = Subset::empty(ground.len());
        let permutations = PermutationGenerator32::new(1);

        Self {
            pattern,
            ground,
            variable,
            empty_produced: !matches_empty,
            subset,
            permutations,
        }
    }

    fn make_next(&self, subset: Vec<Expr>, complement: Vec<Expr>) -> MatchResultList {
        // Attempt to continue to match `f[...]` against `g[...]`.
        let match_equation = MatchResult::MatchEquation(MatchEquation {
            pattern: Expr::from(Normal::new(
                self.pattern.head().clone(),
                &self.pattern.elements()[1..],
            )),
            ground: Expr::from(Normal::new(self.ground.head().clone(), complement)),
        });

        // Create the substitution so long as the pattern was named.
        if let Some(variable) = &self.variable {
            let substitution = MatchResult::Substitution(Substitution {
                variable: variable.clone(),
                ground: Expr::from(Normal::new(sym!(Sequence), subset)),
            });

            return vec![match_equation, substitution];
        }

        vec![match_equation]
    }
}

impl MatchRule for RuleSVEC {
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
            return Some(Self {
                pattern: p.clone(),
                ground: g.clone(),
                variable: variable.cloned(),
                empty_produced: true,
                subset: Subset::full(g.len()),
                permutations: PermutationGenerator32::new(g.len() as u8),
            });
        }

        Some(Self::new(
            p.clone(),
            g.clone(),
            variable.cloned(),
            matches_empty,
        ))
    }
}

impl MatchGenerator for RuleSVEC {
    fn match_equation(&self) -> MatchEquation {
        MatchEquation {
            pattern: Expr::from(self.pattern.clone()),
            ground: Expr::from(self.ground.clone()),
        }
    }
}

impl Iterator for RuleSVEC {
    type Item = MatchResultList;

    fn next(&mut self) -> Option<Self::Item> {
        // If wwe have not yet produced the empty sequence we should do that now.
        if !self.empty_produced {
            self.empty_produced = true;
            return Some(self.make_next(vec![], self.ground.elements().to_vec()));
        }

        // If the current subset is empty we should try and increment it.
        // It is possible to bail early here if ground is `f[]` (i.e. it is empty).
        if self.subset.is_zero() {
            self.subset = self.subset.next()?;
        }

        // Try and get the next permutation for `ground`s elements.
        let permutation = match self.permutations.next() {
            Some(permutation) => permutation,
            None => {
                self.subset = self.subset.next()?;
                self.permutations = PermutationGenerator32::new(self.subset.count_ones() as u8);
                self.permutations.next()?
            }
        };

        // Extract the subset and complement from the current subset
        let (subset, complement) = self.subset.extract(self.ground.elements());

        // Create the next result
        Some(
            self.make_next(
                permutation
                    .map(|idx| subset[idx].clone())
                    .collect::<Vec<_>>(),
                complement,
            ),
        )
    }
}
