use crate::Symbol;
use crate::matching::MatchRule;
use crate::matching::rule_dc::RuleDC;
use crate::matching::rule_df::RuleDF;
use crate::matching::rule_fve::RuleFVE;
use crate::matching::rule_svec::RuleSVEC;
use crate::matching::rule_svef::RuleSVEF;
use crate::matching::rule_t::RuleT;
use crate::matching::rule_ve::RuleVE;
use crate::{
    Context, Expr, MatchEquation, MatchGenerator, MatchResult, MatchResultList, SolutionSet,
    Substitution,
};
use std::collections::HashMap;

type BoxedMatchGenerator = Box<dyn MatchGenerator>;

/// Items that can be pushed onto the match stack.
enum MatchStack {
    /// The match generator responsible for the operations sitting immediately above it on
    /// the stack. Those operations are undone in order to get back to the  match generator
    /// to call `next()`.
    MatchGenerator(BoxedMatchGenerator),

    /// A variable or sequence variable substitution. We only need to record the name of the
    /// variable.
    Substitution(Symbol),

    /// An operation representing pushing matching equations onto the equation stack.
    ProducedMatchEquations(usize),
}

/// Holds the state of the in-process pattern matching attempt.
pub struct Matcher<'c> {
    context: &'c Context,

    /// The match_stack is where operations that change the state are recorded.
    /// Operations are pushed when they are done and popped when they are undone.
    match_stack: Vec<MatchStack>,

    /// The match equations that still need to be solved.
    equation_stack: Vec<MatchEquation>,

    /// The symbol table recording all variable/sequence variable bindings.
    substitutions: SolutionSet,
}

impl<'c> Matcher<'c> {
    pub fn new(pattern: impl Into<Expr>, subject: impl Into<Expr>, context: &'c Context) -> Self {
        Self {
            context,
            match_stack: Vec::new(),
            equation_stack: vec![MatchEquation {
                pattern: pattern.into(),
                ground: subject.into(),
            }],
            substitutions: HashMap::new(),
        }
    }

    /// Check which rule applies to the active match equation, creates the match generator for that
    /// rule, and pushes the match generator onto the match stack.
    fn select_rule(&mut self) -> Option<BoxedMatchGenerator> {
        // TODO: Substitute bound variables with their values. This probably should NOT happen here
        //  though as we would otherwise be rebuilding `match_equation` on every evaluation step.
        let match_equation = match self.equation_stack.pop() {
            Some(match_equation) => match_equation,
            None => return None,
        };

        if let Some(rule) = RuleT::try_rule(&match_equation) {
            return Some(Box::new(rule));
        }

        if let Some(rule) = RuleVE::try_rule(&match_equation) {
            return Some(Box::new(rule));
        }

        if let Some(rule) = RuleFVE::try_rule(&match_equation) {
            return Some(Box::new(rule));
        }

        // At this point we next need to perform destructuring type operations. This requests both
        // `pattern` and `ground` to be expressions.
        if let (Some(p), Some(g)) = (
            match_equation.pattern.try_normal(),
            match_equation.ground.try_normal(),
        ) {
            // TODO: Need to be able to match `f[...][...]` to `g[...][...]`. Will need to add
            //  rules to destructure the head in this case. This should be recursive to support
            //  operations like `f[...][...][...]` etc.

            // Attempting to match `f[...]` with `g[...]' where `f` and `g` are symbols and match.
            if let (Some(phead), Some(ghead)) = (p.try_head_symbol(), g.try_head_symbol()) {
                if phead != ghead {
                    self.equation_stack.push(match_equation);
                    return None;
                }

                let ground_attributes = self.context.get_attributes(ghead);

                return match (
                    ground_attributes.commutative(),
                    ground_attributes.associative(),
                ) {
                    // Rules for Free Functions (neither associative nor commutative).
                    (false, false) => {
                        if let Some(rule) = RuleDF::try_rule(&match_equation) {
                            return Some(Box::new(rule));
                        }

                        if let Some(rule) = RuleSVEF::try_rule(&match_equation) {
                            return Some(Box::new(rule));
                        }

                        self.equation_stack.push(match_equation);
                        None
                    }

                    // Rules for commutative functions.
                    (true, false) => {
                        if let Some(rule) = RuleDC::try_rule(&match_equation) {
                            return Some(Box::new(rule));
                        }

                        if let Some(rule) = RuleSVEC::try_rule(&match_equation) {
                            return Some(Box::new(rule));
                        }

                        self.equation_stack.push(match_equation);
                        None
                    }

                    // Rules for associative functions.
                    (false, true) => {
                        // if let Some(rule) = RuleSVEA::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleFVEA::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleIVEA::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleDecA::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }

                        self.equation_stack.push(match_equation);
                        None
                    }

                    // Rules for associative-commutative symbols.
                    (true, true) => {
                        // if let Some(rule) = RuleSVEAC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleFVEAC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleIVEAC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }
                        //
                        // if let Some(rule) = RuleDecAC::try_rule(&match_equation) {
                        //     return Some(Box::new(rule));
                        // }

                        self.equation_stack.push(match_equation);
                        None
                    }
                };
            }
        }

        self.equation_stack.push(match_equation);
        None
    }

    /// Undoes the effects of the last call to `next()`, removes the match generator, and
    /// restores any match equation corresponding to the match generator, if any.
    fn backtrack(&mut self) {
        let match_generator = self.undo();
        self.equation_stack.push(match_generator.match_equation());
    }

    /// Undoes the effects of the last call to `next()` and, for convenience, returns the
    /// responsible match generator.
    /// Upon return the `MatchGenerator` will be active, i.e. on top of the match stack.
    fn undo(&mut self) -> BoxedMatchGenerator {
        loop {
            match self.match_stack.pop().unwrap() {
                // Leave the match generator on top of the match stack.
                // Don't restore its match equation.
                MatchStack::MatchGenerator(match_generator) => {
                    return match_generator;
                }

                // Remove the substitution
                MatchStack::Substitution(expression) => {
                    self.substitutions.remove(&expression);
                }

                // Remove the produced equations
                MatchStack::ProducedMatchEquations(added) => {
                    let new_length = self.equation_stack.len() - added;
                    self.equation_stack.truncate(new_length);
                }
            }
        }
    }

    fn process_match_list(&mut self, mut results: MatchResultList) {
        let mut equation_count: usize = 0;

        for result in results.drain(..) {
            match result {
                MatchResult::Substitution(substitution) => {
                    self.push_substitution(substitution);
                }

                MatchResult::MatchEquation(match_equation) => {
                    self.equation_stack.push(match_equation);
                    equation_count += 1;
                }
            }
        }

        if equation_count > 0 {
            self.push_match_equations(equation_count);
        }
    }

    fn push_rule(&mut self, generator: BoxedMatchGenerator) {
        self.match_stack.push(MatchStack::MatchGenerator(generator));
    }

    fn push_substitution(&mut self, Substitution { variable, ground }: Substitution) {
        self.substitutions.insert(variable.clone(), ground.clone());

        self.match_stack
            .push(MatchStack::Substitution(variable.clone()));
    }

    fn push_match_equations(&mut self, equation_count: usize) {
        self.match_stack
            .push(MatchStack::ProducedMatchEquations(equation_count));
    }
}

impl<'c> Iterator for Matcher<'c> {
    type Item = SolutionSet;

    fn next(&mut self) -> Option<Self::Item> {
        // If the last match was successful, the equation stack will be empty. But there could be
        // more solutions possible, in which case backtracking will put equations back on the stack.
        if self.equation_stack.is_empty() && self.match_stack.is_empty() {
            return None;
        }

        'step1: loop {
            // Attempt to select a rule to apply
            match self.select_rule() {
                // We have found a rule to apply.
                Some(match_generator) => {
                    // Push the `MatchGenerator` onto the match stack.
                    // It is now the active `MatchGenerator`.
                    self.push_rule(match_generator);
                }

                // No rule applies.
                None => {
                    // If the match stack is empty, halt with failure.
                    if self.match_stack.is_empty() {
                        return None;
                    }

                    // If there is an active match generator on top of the matcher stack, undo the
                    // actions of the last match generated from this match generator.
                    let match_generator = self.undo();

                    // But retain the active match generator.
                    self.push_rule(match_generator);
                }
            }

            // Apply the rule.
            'step2: loop {
                match self.match_stack.last_mut() {
                    None => {
                        panic!("Expected value on the match stack. Found nothing.")
                    }

                    Some(MatchStack::MatchGenerator(match_generator)) => {
                        match match_generator.next() {
                            // Found a match.
                            Some(results) => {
                                self.process_match_list(results);

                                // Succeed if the equation stack is empty.
                                if self.equation_stack.is_empty() {
                                    return Some(self.substitutions.clone());
                                }

                                continue 'step1;
                            }

                            // No matches found.
                            // Backtrack to the previous matcher.
                            None => {
                                self.backtrack();

                                // Fail if there is no previous matcher to backtrack to.
                                if self.match_stack.is_empty() {
                                    return None;
                                }

                                let match_generator = self.undo();
                                self.push_rule(match_generator);

                                continue 'step2;
                            }
                        }
                    }

                    Some(MatchStack::Substitution(substitution)) => {
                        panic!(
                            "Expected a MatchGenerator. Found substitution: '{}'.",
                            substitution
                        )
                    }

                    Some(MatchStack::ProducedMatchEquations(me)) => {
                        panic!(
                            "Expected a MatchGenerator. Found {} produced match equations.",
                            me
                        )
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    macro_rules! assert_matches {
        // No more solutions expected
        ($matcher:ident, []) => {
            assert_eq!($matcher.next(), None);
        };

        // One or more solutions
        (
            $matcher:ident,
            [
                { $($name:expr => $value:expr),* $(,)? }
                $(, { $($rest_name:expr => $rest_value:expr),* })*
                $(,)?
            ]
        ) => {
            assert_eq!(
                $matcher.next(),
                Some(std::collections::HashMap::from([
                    $( (crate::Symbol::new($name), crate::parse($value).unwrap()) ),*
                ]))
            );

            assert_matches!(
                $matcher,
                [ $( { $($rest_name => $rest_value),* } ),* ]
            );
        };
    }

    macro_rules! matcher_test {
        ($name:ident, $pattern:expr, $ground:expr, $matches:tt) => {
            #[test]
            fn $name() -> () {
                let mut context = Context::new();

                context
                    .set_attributes(
                        &Symbol::new("fc"),
                        crate::Attributes::from(crate::Attribute::Commutative),
                    )
                    .unwrap();

                let mut matcher =
                    Matcher::new(parse($pattern).unwrap(), parse($ground).unwrap(), &context);

                assert_matches!(matcher, $matches);
            }
        };
    }

    // Unsolvable
    matcher_test!(mismatched_strings, "\"abc\"", "\"def\"", []);
    matcher_test!(mismatched_integers, "123", "456", []);
    matcher_test!(mismatched_reals, "2.5", "3.5", []);
    matcher_test!(mismatched_symbol, "abc", "def", []);
    matcher_test!(
        blank_sequence_does_not_match_empty_sequence,
        "__",
        "Sequence[]",
        []
    );

    // Solvable with no substitutions
    matcher_test!(matching_strings, "\"abc\"", "\"abc\"", [{}]);
    matcher_test!(matching_integers, "123", "123", [{}]);
    matcher_test!(matching_reals, "2.5", "2.5", [{}]);
    matcher_test!(matching_symbols, "abc", "abc", [{}]);

    // Unnamed blank
    matcher_test!(blank_matches_string, "_", "\"abc\"", [{}]);
    matcher_test!(blank_matches_integer, "_", "123", [{}]);
    matcher_test!(blank_matches_real, "_", "2.5", [{}]);
    matcher_test!(blank_matches_symbol, "_", "abc", [{}]);
    matcher_test!(blank_matches_expression, "_", "f[a, b, c]", [{}]);
    matcher_test!(blank_matches_empty_sequence, "_", "Sequence[]", [{}]);
    matcher_test!(blank_matches_sequence_length_1, "_", "Sequence[a]", [{}]);
    matcher_test!(blank_matches_sequence_length_2, "_", "Sequence[a, b]", [{}]);
    matcher_test!(
        blank_matches_sequence_length_3,
        "_",
        "Sequence[a, b, c]",
        [{}]
    );

    // Named blank
    matcher_test!(named_blank_matches_string, "x_", "\"abc\"", [{ "x" => "\"abc\"" }]);
    matcher_test!(named_blank_matches_integer, "x_", "123", [{ "x" => "123" }]);
    matcher_test!(named_blank_matches_real, "x_", "2.5", [{ "x" => "2.5" }]);
    matcher_test!(named_blank_matches_symbol, "x_", "abc", [{ "x" => "abc" }]);
    matcher_test!(named_blank_matches_expression, "x_", "f[a, b, c]", [{ "x" => "f[a, b, c]" }]);
    matcher_test!(named_blank_matches_empty_sequence, "x_", "Sequence[]", [{ "x" => "Sequence[]" }]);
    matcher_test!(named_blank_matches_sequence_length_1, "x_", "Sequence[a]", [{ "x" => "Sequence[a]" }]);
    matcher_test!(named_blank_matches_sequence_length_2, "x_", "Sequence[a, b]", [{ "x" => "Sequence[a, b]" }]);
    matcher_test!(named_blank_matches_sequence_length_3, "x_", "Sequence[a, b, c]", [{ "x" => "Sequence[a, b, c]" }]);

    // Unnamed blank sequence
    matcher_test!(
        blank_sequence_matches_sequence_length_1,
        "__",
        "Sequence[a]",
        [{}]
    );
    matcher_test!(
        blank_sequence_matches_sequence_length_2,
        "__",
        "Sequence[a, b]",
        [{}]
    );
    matcher_test!(
        blank_sequence_matches_sequence_length_3,
        "__",
        "Sequence[a, b, c]",
        [{}]
    );

    // Named blank sequence
    matcher_test!(named_blank_sequence_matches_sequence_length_1, "xs__", "Sequence[a]", [{ "xs" => "Sequence[a]" }]);
    matcher_test!(named_blank_sequence_matches_sequence_length_2, "xs__", "Sequence[a, b]", [{ "xs" => "Sequence[a, b]" }]);
    matcher_test!(named_blank_sequence_matches_sequence_length_3, "xs__", "Sequence[a, b, c]", [{ "xs" => "Sequence[a, b, c]" }]);

    // Unnamed blank null sequence
    matcher_test!(
        blank_null_sequence_matches_empty_sequence,
        "___",
        "Sequence[]",
        [{}]
    );
    matcher_test!(
        blank_null_sequence_matches_sequence_length_1,
        "___",
        "Sequence[a]",
        [{}]
    );
    matcher_test!(
        blank_null_sequence_matches_sequence_length_2,
        "___",
        "Sequence[a, b]",
        [{}]
    );
    matcher_test!(
        blank_null_sequence_matches_sequence_length_3,
        "___",
        "Sequence[a, b, c]",
        [{}]
    );

    // Named blank null sequence
    matcher_test!(named_blank_null_sequence_matches_empty_sequence, "xs___", "Sequence[]", [{ "xs" => "Sequence[]" }]);
    matcher_test!(named_blank_null_sequence_matches_sequence_length_1, "xs___", "Sequence[a]", [{ "xs" => "Sequence[a]" }]);
    matcher_test!(named_blank_null_sequence_matches_sequence_length_2, "xs___", "Sequence[a, b]", [{ "xs" => "Sequence[a, b]" }]);
    matcher_test!(named_blank_null_sequence_matches_sequence_length_3, "xs___", "Sequence[a, b, c]", [{ "xs" => "Sequence[a, b, c]" }]);

    // TODO: Tests for FVE `f_[a, b, c]` and `f[a, b, c]`.

    mod free_functions {
        use super::*;

        // Unsolvable
        matcher_test!(mismatched_expression_heads, "f[a, b, c]", "g[a, b ,c]", []);
        matcher_test!(
            mismatched_expression_elements,
            "f[a, b, c]",
            "f[d, e, f]",
            []
        );
        matcher_test!(blank_with_mismatch, "f[_, b]", "f[a, c]", []);
        matcher_test!(unmatchable_blank_sequence, "f[__]", "f[]", []);

        // Trivial cases
        matcher_test!(exact_match, "f[a, b, c]", "f[a, b, c]", [{}]);

        // Unnamed blanks
        matcher_test!(blank_in_first_element, "f[_, b, c]", "f[a, b, c]", [{}]);
        matcher_test!(blank_in_second_element, "f[a, _, c]", "f[a, b, c]", [{}]);
        matcher_test!(blank_in_third_element, "f[a, b, _]", "f[a, b, c]", [{}]);

        // Single named blank
        matcher_test!(named_blank_in_first_element, "f[x_, b, c]", "f[a, b, c]", [{ "x" => "a" }]);
        matcher_test!(named_blank_in_second_element, "f[a, x_, c]", "f[a, b, c]", [{ "x" => "b" }]);
        matcher_test!(named_blank_in_third_element, "f[a, b, x_]", "f[a, b, c]", [{ "x" => "c" }]);

        // Multiple unnamed blanks
        matcher_test!(
            blank_in_first_and_second_element,
            "f[_, _, c]",
            "f[a, b, c]",
            [{}]
        );
        matcher_test!(
            blank_in_second_and_third_element,
            "f[a, _, _]",
            "f[a, b, c]",
            [{}]
        );
        matcher_test!(
            blank_in_first_and_third_element,
            "f[_, b, _]",
            "f[a, b, c]",
            [{}]
        );
        matcher_test!(blank_in_all_elements, "f[_, _, _]", "f[a, b, c]", [{}]);

        // Multiple named blanks
        matcher_test!(named_blank_in_first_and_second_elements, "f[x_, y_, c]", "f[a, b, c]", [{ "x" => "a", "y" => "b" }]);
        matcher_test!(named_blank_in_first_and_third_elements, "f[x_, b, y_]", "f[a, b, c]", [{ "x" => "a", "y" => "c" }]);
        matcher_test!(named_blank_in_second_and_third_elements, "f[a, x_, y_]", "f[a, b, c]", [{ "x" => "b", "y" => "c" }]);

        // Multiple named blank sequences
        matcher_test!(multiple_blank_sequences, "f[xs__, ys__]", "f[a, b, c]", [
            {"xs" => "Sequence[a]", "ys" => "Sequence[b, c]"},
            {"xs" => "Sequence[a, b]", "ys" => "Sequence[c]"},
        ]);

        // Multiple named blank null sequences
        matcher_test!(multiple_blank_null_sequences, "f[xs___, ys___]", "f[a, b, c]", [
            {"xs" => "Sequence[]", "ys" => "Sequence[a, b,c ]"},
            {"xs" => "Sequence[a]", "ys" => "Sequence[b, c]"},
            {"xs" => "Sequence[a, b]", "ys" => "Sequence[c]"},
            {"xs" => "Sequence[a, b, c]", "ys" => "Sequence[]"},
        ]);
    }

    mod commutative {
        use super::*;
        use crate::{Attribute, Attributes};
        use test_case::test_case;

        fn create_context() -> Context {
            let mut context = Context::new();

            context
                .set_attributes(&Symbol::new("f"), Attributes::from(Attribute::Commutative))
                .unwrap();

            context
        }

        #[test_case("f[a, b, c]", "g[a, b, c]" ; "mismatched expression heads")]
        #[test_case("f[a, b]", "f[a, c]" ; "unmatchable parameter")]
        #[test_case("f[a, b]", "f[a, b, c]" ; "extra parameter")]
        #[test_case("f[__]", "f[]" ; "cannot match __ to empty sequence")]
        fn unmatchable_expressions_gives_no_solutions(pattern: &str, ground: &str) {
            let context = Context::new();

            let mut matcher =
                Matcher::new(parse(pattern).unwrap(), parse(ground).unwrap(), &context);

            assert_matches!(matcher, []);
        }

        #[test_case("f[a, b, c]", "f[a, b, c]" ; "exact match")]
        #[test_case("f[a, b, c]", "f[c, b, a]" ; "matches regardless of argument order")]
        #[test_case("f[a, _, c]", "f[c, _, a]" ; "matches blank regardless of argument order")]
        fn handles_matches_with_single_empty_solution(pattern: &str, ground: &str) {
            let context = create_context();

            let mut matcher =
                Matcher::new(parse(pattern).unwrap(), parse(ground).unwrap(), &context);

            assert_matches!(matcher, [{}]);
        }

        #[test_case("f[x_, b, c]", "f[a, b, c]", "a" ; "in first element")]
        #[test_case("f[a, x_, c]", "f[a, b, c]", "b" ; "in second element")]
        #[test_case("f[a, b, x_]", "f[a, b, c]", "c" ; "in third element")]
        fn handles_single_blank_named_variable(pattern: &str, ground: &str, x: &str) {
            let context = create_context();

            let mut matcher =
                Matcher::new(parse(pattern).unwrap(), parse(ground).unwrap(), &context);

            assert_matches!(matcher, [{"x" => x}]);
        }

        #[test_case("f[x_, y_, c]", "f[a, b, c]", "a", "b" ; "in first and second elements")]
        #[test_case("f[x_, b, y_]", "f[a, b, c]", "a", "c" ; "in first and third elements")]
        #[test_case("f[a, y_, x_]", "f[a, b, c]", "c", "b" ; "in second and third elements")]
        fn handles_two_blank_named_variables(pattern: &str, ground: &str, x: &str, y: &str) {
            let context = create_context();

            let mut matcher =
                Matcher::new(parse(pattern).unwrap(), parse(ground).unwrap(), &context);

            assert_matches!(matcher, [
                {"x" => x, "y" => y},
                {"x" => y, "y" => x},
            ]);
        }

        #[test]
        fn handles_multiple_blank_sequence_variables() {
            let context = create_context();

            let mut matcher = Matcher::new(
                parse("f[xs__, ys__]").unwrap(),
                parse("f[a, b, c]").unwrap(),
                &context,
            );

            assert_matches!(matcher, [
                {"xs" => "Sequence[a]", "ys" => "Sequence[b, c]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[c, b]"},
                {"xs" => "Sequence[b]", "ys" => "Sequence[a, c]"},
                {"xs" => "Sequence[b]", "ys" => "Sequence[c, a]"},
                {"xs" => "Sequence[a, b]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[b, a]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[c]", "ys" => "Sequence[a, b]"},
                {"xs" => "Sequence[c]", "ys" => "Sequence[b, a]"},
                {"xs" => "Sequence[a, c]", "ys" => "Sequence[b]"},
                {"xs" => "Sequence[c, a]", "ys" => "Sequence[b]"},
                {"xs" => "Sequence[b, c]", "ys" => "Sequence[a]"},
                {"xs" => "Sequence[c, b]", "ys" => "Sequence[a]"},
            ]);
        }

        // #[test]
        // fn handles_multiple_blank_null_sequence_variables() {
        //     let context = create_context();
        //
        //     let mut matcher = Matcher::new(
        //         parse("f[xs___, ys___]").unwrap(),
        //         parse("f[a, b]").unwrap(),
        //         &context,
        //     );
        //
        //     assert_eq!(
        //         matcher.next(),
        //         Some(HashMap::from([
        //             (Symbol::new("xs"), parse("Sequence[]").unwrap()),
        //             (Symbol::new("ys"), parse("Sequence[a, b]").unwrap())
        //         ]))
        //     );
        //     assert_eq!(
        //         matcher.next(),
        //         Some(HashMap::from([
        //             (Symbol::new("xs"), parse("Sequence[a]").unwrap()),
        //             (Symbol::new("ys"), parse("Sequence[b]").unwrap())
        //         ]))
        //     );
        //     assert_eq!(
        //         matcher.next(),
        //         Some(HashMap::from([
        //             (Symbol::new("xs"), parse("Sequence[a, b]").unwrap()),
        //             (Symbol::new("ys"), parse("Sequence[]").unwrap())
        //         ]))
        //     );
        //     assert_eq!(matcher.next(), None);
        // }
    }
}
