use crate::matching::function_application::{AFAGenerator, FunctionApplicationGenerator};
use crate::matching::permutations::PermutationGenerator32;
use crate::{Expr, Normal};

/// Associative-Commutative Function Application Generator.
pub struct AFACGenerator {
    function: Normal,
    afa_generator: AFAGenerator,
    permutations: PermutationGenerator32,
}

impl FunctionApplicationGenerator for AFACGenerator {
    fn new(function: Normal) -> Self {
        // The first permutation is the identity, so just clone the function.
        let mut permutations = PermutationGenerator32::new(function.len() as u8);
        permutations.next();

        Self {
            function: function.clone(),
            afa_generator: AFAGenerator::new(function.clone()),
            permutations,
        }
    }
}

impl Iterator for AFACGenerator {
    type Item = Vec<Expr>;

    fn next(&mut self) -> Option<Vec<Expr>> {
        match self.afa_generator.next() {
            // Generator is empty - determine the next permutation to use.
            None => match self.permutations.next() {
                // There are no more permutations, and the current generator is empty. We have now
                // exhausted all possible applications.
                None => None,

                // Start generating more function applications using a newly ordered set of the
                // function elements.
                Some(permutation) => {
                    let permuted_function = Normal::new(
                        self.function.head().clone(),
                        permutation
                            .map(|n| self.function.elements()[n].clone())
                            .collect::<Vec<_>>(),
                    );

                    self.afa_generator = AFAGenerator::new(permuted_function);
                    self.afa_generator.next()
                }
            },

            // Current generator is continuing to produce results.
            Some(results) => Some(results),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn mk(exprs: &[&str]) -> Option<Vec<Expr>> {
        Some(exprs.iter().map(|e| parse(e).unwrap()).collect::<Vec<_>>())
    }

    #[test]
    fn generates_all_afa_applications() {
        let f = parse("f[a, b, c]").unwrap();
        let f_normal = f.try_normal().unwrap().clone();

        let mut afac_gen = AFACGenerator::new(f_normal);

        // Permutation ABC
        assert_eq!(afac_gen.next(), mk(&["a", "b", "c"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "b", "c"]));
        assert_eq!(afac_gen.next(), mk(&["a", "f[b]", "c"]));
        assert_eq!(afac_gen.next(), mk(&["a", "b", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "f[b]", "c"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "b", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["a", "f[b]", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "f[b]", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a, b]", "c"]));
        assert_eq!(afac_gen.next(), mk(&["f[a, b]", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["a", "f[b, c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "f[b, c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a, b, c]"]));

        // Permutation ACB
        assert_eq!(afac_gen.next(), mk(&["a", "c", "b"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "c", "b"]));
        assert_eq!(afac_gen.next(), mk(&["a", "f[c]", "b"]));
        assert_eq!(afac_gen.next(), mk(&["a", "c", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "f[c]", "b"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "c", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["a", "f[c]", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "f[c]", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a, c]", "b"]));
        assert_eq!(afac_gen.next(), mk(&["f[a, c]", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["a", "f[c, b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a]", "f[c, b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[a, c, b]"]));

        // Permutation BAC
        assert_eq!(afac_gen.next(), mk(&["b", "a", "c"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "a", "c"]));
        assert_eq!(afac_gen.next(), mk(&["b", "f[a]", "c"]));
        assert_eq!(afac_gen.next(), mk(&["b", "a", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "f[a]", "c"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "a", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["b", "f[a]", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "f[a]", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b, a]", "c"]));
        assert_eq!(afac_gen.next(), mk(&["f[b, a]", "f[c]"]));
        assert_eq!(afac_gen.next(), mk(&["b", "f[a, c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "f[a, c]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b, a, c]"]));

        // Permutation BCA
        assert_eq!(afac_gen.next(), mk(&["b", "c", "a"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "c", "a"]));
        assert_eq!(afac_gen.next(), mk(&["b", "f[c]", "a"]));
        assert_eq!(afac_gen.next(), mk(&["b", "c", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "f[c]", "a"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "c", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["b", "f[c]", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "f[c]", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b, c]", "a"]));
        assert_eq!(afac_gen.next(), mk(&["f[b, c]", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["b", "f[c, a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b]", "f[c, a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[b, c, a]"]));

        // Permutation CAB
        assert_eq!(afac_gen.next(), mk(&["c", "a", "b"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "a", "b"]));
        assert_eq!(afac_gen.next(), mk(&["c", "f[a]", "b"]));
        assert_eq!(afac_gen.next(), mk(&["c", "a", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "f[a]", "b"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "a", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["c", "f[a]", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "f[a]", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c, a]", "b"]));
        assert_eq!(afac_gen.next(), mk(&["f[c, a]", "f[b]"]));
        assert_eq!(afac_gen.next(), mk(&["c", "f[a, b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "f[a, b]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c, a, b]"]));

        // Permutation CBA
        assert_eq!(afac_gen.next(), mk(&["c", "b", "a"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "b", "a"]));
        assert_eq!(afac_gen.next(), mk(&["c", "f[b]", "a"]));
        assert_eq!(afac_gen.next(), mk(&["c", "b", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "f[b]", "a"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "b", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["c", "f[b]", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "f[b]", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c, b]", "a"]));
        assert_eq!(afac_gen.next(), mk(&["f[c, b]", "f[a]"]));
        assert_eq!(afac_gen.next(), mk(&["c", "f[b, a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c]", "f[b, a]"]));
        assert_eq!(afac_gen.next(), mk(&["f[c, b, a]"]));

        assert_eq!(afac_gen.next(), None);
    }
}
