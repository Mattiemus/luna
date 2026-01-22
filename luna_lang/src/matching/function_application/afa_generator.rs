use crate::matching::function_application::FunctionApplicationGenerator;
use crate::{Expr, Normal};
use std::cmp::min;
use crate::matching::subsets::next_subset;

/// Associative Function Application Generator.
pub struct AFAGenerator {
    function: Normal,

    /// A bit vector encoding which terms are outside any function application.
    /// This flag also indicates exhaustion.
    singleton_state: u32,

    /// A bit vector encoding how terms are grouped into function applications.
    application_state: u32,
}

impl FunctionApplicationGenerator for AFAGenerator {
    fn new(function: Normal) -> Self {
        Self {
            function,
            singleton_state: 0,
            application_state: 0,
        }
    }
}

impl Iterator for AFAGenerator {
    type Item = Vec<Expr>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = min(self.function.len(), 32);

        if n == 0 || self.singleton_state == u32::MAX {
            return None;
        }

        let mut last_boundary_position = 0;
        let mut singleton_count = 0;

        let mut result_sequence = Vec::with_capacity(n);

        for position in 1..=n {
            if position == n || ((1 << (position - 1)) & !self.application_state) > 0 {
                if position - last_boundary_position > 1 {
                    let new_function = Expr::from(Normal::new(
                        self.function.head().clone(),
                        &self.function.elements()[last_boundary_position..position],
                    ));

                    result_sequence.push(new_function);
                } else {
                    if (1 << singleton_count) & self.singleton_state > 0 {
                        let new_function = Expr::from(Normal::new(
                            self.function.head().clone(),
                            [self.function.element(position - 1).clone()],
                        ));

                        result_sequence.push(new_function);
                    } else {
                        result_sequence.push(self.function.element(position - 1).clone());
                    }

                    singleton_count += 1;
                }

                last_boundary_position = position;
            }
        }

        if self.singleton_state != u32::MAX {
            if let Some(next_singleton_state) =
                next_subset(singleton_count as u32, self.singleton_state)
            {
                self.singleton_state = next_singleton_state;
            } else {
                if let Some(next_application_state) =
                    next_subset((n - 1) as u32, self.application_state)
                {
                    self.singleton_state = 0;
                    self.application_state = next_application_state;
                } else {
                    self.singleton_state = u32::MAX;
                }
            }
        }

        Some(result_sequence)
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

        let mut afa_gen = AFAGenerator::new(f_normal);

        assert_eq!(afa_gen.next(), mk(&["a", "b", "c"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "b", "c"]));
        assert_eq!(afa_gen.next(), mk(&["a", "f[b]", "c"]));
        assert_eq!(afa_gen.next(), mk(&["a", "b", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "f[b]", "c"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "b", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["a", "f[b]", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "f[b]", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a, b]", "c"]));
        assert_eq!(afa_gen.next(), mk(&["f[a, b]", "f[c]"]));
        assert_eq!(afa_gen.next(), mk(&["a", "f[b, c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a]", "f[b, c]"]));
        assert_eq!(afa_gen.next(), mk(&["f[a, b, c]"]));
        assert_eq!(afa_gen.next(), None);
    }
}
