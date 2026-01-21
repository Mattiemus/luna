use crate::Symbol;
use crate::matching::MatchRule;
use crate::matching::rule_dc::RuleDC;
use crate::matching::rule_dnc::RuleDNC;
use crate::matching::rule_fve::RuleFVE;
use crate::matching::rule_sve::{RuleSVEA, RuleSVEAC};
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
    pub fn new(pattern: Expr, ground: Expr, context: &'c Context) -> Self {
        Self {
            context,
            match_stack: Vec::new(),
            equation_stack: vec![MatchEquation { pattern, ground }],
            substitutions: HashMap::new(),
        }
    }

    /// Check which rule applies to the active match equation, creates the match generator for that
    /// rule, and pushes the match generator onto the match stack.
    fn select_rule(&mut self) -> Option<BoxedMatchGenerator> {
        // TODO: Substitute bound variables with their values for the *pattern*.
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
                        if let Some(rule) = RuleDNC::try_rule(&match_equation) {
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
                        if let Some(rule) = RuleSVEA::try_rule(&match_equation) {
                            return Some(Box::new(rule));
                        }

                        // if let Some(rule) = RuleFVEA::try_rule(&match_equation) {
                        // if let Some(rule) = RuleIVEA::try_rule(&match_equation) {

                        if let Some(rule) = RuleDNC::try_rule(&match_equation) {
                            return Some(Box::new(rule));
                        }

                        self.equation_stack.push(match_equation);
                        None
                    }

                    // Rules for associative-commutative symbols.
                    (true, true) => {
                        if let Some(rule) = RuleSVEAC::try_rule(&match_equation) {
                            return Some(Box::new(rule));
                        }

                        // if let Some(rule) = RuleFVEAC::try_rule(&match_equation) {
                        // if let Some(rule) = RuleIVEAC::try_rule(&match_equation) {

                        if let Some(rule) = RuleDC::try_rule(&match_equation) {
                            return Some(Box::new(rule));
                        }

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
                MatchResult::Substitution(Substitution { variable, ground }) => {
                    self.substitutions.insert(variable.clone(), ground.clone());

                    self.match_stack
                        .push(MatchStack::Substitution(variable.clone()));
                }

                MatchResult::MatchEquation(match_equation) => {
                    self.equation_stack.push(match_equation);
                    equation_count += 1;
                }
            }
        }

        if equation_count > 0 {
            self.match_stack
                .push(MatchStack::ProducedMatchEquations(equation_count));
        }
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
                    self.match_stack
                        .push(MatchStack::MatchGenerator(match_generator));
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
                    self.match_stack
                        .push(MatchStack::MatchGenerator(match_generator));
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
                                self.match_stack
                                    .push(MatchStack::MatchGenerator(match_generator));

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

                context
                    .set_attributes(
                        &Symbol::new("fa"),
                        crate::Attributes::from(crate::Attribute::Associative),
                    )
                    .unwrap();

                context
                    .set_attributes(
                        &Symbol::new("fac"),
                        crate::Attribute::Associative + crate::Attribute::Commutative,
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

    // Blanks in head
    matcher_test!(unnamed_blank_in_head, "_[a, b, c]", "abc[a, b, c]", [{}]);
    matcher_test!(named_blank_in_head, "x_[a, b, c]", "abc[a, b, c]", [{ "x" => "abc" }]);

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

        // Unmatchable cases
        matcher_test!(mismatched_heads, "fc[a, b, c]", "g[a, b, c]", []);
        matcher_test!(unmatchable_param, "fc[a, b]", "fc[a, c]", []);
        matcher_test!(extra_param, "fc[a, b]", "fc[a, b, c]", []);
        matcher_test!(blank_seq_empty, "fc[__]", "fc[]", []);

        // Exact and order-independent matches
        matcher_test!(blank_null_seq_empty, "fc[___]", "fc[]", [{}]);
        matcher_test!(exact_match, "fc[a, b, c]", "fc[a, b, c]", [{}]);
        matcher_test!(order_independent_match, "fc[a, b, c]", "fc[c, b, a]", [{}]);
        matcher_test!(blank_order_independent, "fc[a, _, c]", "fc[c, _, a]", [{}]);

        // Single named blank variable
        matcher_test!(named_blank_first, "fc[x_, b, c]", "fc[a, b, c]", [{ "x" => "a" }]);
        matcher_test!(named_blank_second, "fc[a, x_, c]", "fc[a, b, c]", [{ "x" => "b" }]);
        matcher_test!(named_blank_third, "fc[a, b, x_]", "fc[a, b, c]", [{ "x" => "c" }]);

        // Two named blank variables
        matcher_test!(
            two_named_blanks_1,
            "fc[x_, y_, c]",
            "fc[a, b, c]",
            [
                { "x" => "a", "y" => "b" },
                { "x" => "b", "y" => "a" }
            ]
        );

        matcher_test!(
            two_named_blanks_2,
            "fc[x_, b, y_]",
            "fc[a, b, c]",
            [
                { "x" => "a", "y" => "c" },
                { "x" => "c", "y" => "a" }
            ]
        );

        matcher_test!(
            two_named_blanks_3,
            "fc[a, x_, y_]",
            "fc[a, b, c]",
            [
                { "x" => "b", "y" => "c" },
                { "x" => "c", "y" => "b" }
            ]
        );

        // Multiple blank sequence variables
        matcher_test!(
            multiple_blank_sequences,
            "fc[xs__, ys__]",
            "fc[a, b, c]",
            [
                {"xs" => "Sequence[a]", "ys" => "Sequence[b, c]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[c, b]"},
                {"xs" => "Sequence[b]", "ys" => "Sequence[a, c]"},
                {"xs" => "Sequence[b]", "ys" => "Sequence[c, a]"},
                {"xs" => "Sequence[c]", "ys" => "Sequence[a, b]"},
                {"xs" => "Sequence[c]", "ys" => "Sequence[b, a]"},
                {"xs" => "Sequence[a, b]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[b, a]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[a, c]", "ys" => "Sequence[b]"},
                {"xs" => "Sequence[c, a]", "ys" => "Sequence[b]"},
                {"xs" => "Sequence[b, c]", "ys" => "Sequence[a]"},
                {"xs" => "Sequence[c, b]", "ys" => "Sequence[a]"}
            ]
        );

        // Multiple blank null sequence variables
        matcher_test!(
            multiple_blank_null_sequences,
            "fc[xs___, ys___]",
            "fc[a, b, c]",
            [
                {"xs" => "Sequence[]", "ys" => "Sequence[a, b, c]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[a, c, b]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[b, a, c]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[b, c, a]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[c, a, b]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[c, b, a]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[b, c]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[c, b]"},
                {"xs" => "Sequence[b]", "ys" => "Sequence[a, c]"},
                {"xs" => "Sequence[b]", "ys" => "Sequence[c, a]"},
                {"xs" => "Sequence[c]", "ys" => "Sequence[a, b]"},
                {"xs" => "Sequence[c]", "ys" => "Sequence[b, a]"},
                {"xs" => "Sequence[a, b]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[b, a]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[a, c]", "ys" => "Sequence[b]"},
                {"xs" => "Sequence[c, a]", "ys" => "Sequence[b]"},
                {"xs" => "Sequence[b, c]", "ys" => "Sequence[a]"},
                {"xs" => "Sequence[c, b]", "ys" => "Sequence[a]"},
                {"xs" => "Sequence[a, b, c]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[a, c, b]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[b, a, c]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[b, c, a]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[c, a, b]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[c, b, a]", "ys" => "Sequence[]"},
            ]
        );
    }

    mod associative {
        use super::*;

        // Unmatchable cases
        matcher_test!(mismatched_heads, "fa[a, b, c]", "g[a, b, c]", []);
        matcher_test!(extra_param, "fa[a, b]", "fa[a, b, c]", []);
        matcher_test!(blank_seq_empty, "fa[__]", "fa[]", []);

        // Exact and application-independent matches
        matcher_test!(blank_null_seq_empty, "fa[___]", "fa[]", [{}]);
        matcher_test!(exact_match, "fa[a, b, c]", "fa[a, b, c]", [{}]);
        // TODO: matcher_test!(application_independent_match_1, "fa[fa[a, b], c]", "fa[a, b, c]", [{}]);
        // TODO: matcher_test!(application_independent_match_2, "fa[a, fa[b, c]]", "fa[a, b, c]", [{}]);

        // Multiple blank sequence variables
        matcher_test!(
            multiple_blank_sequences,
            "fa[xs__, ys__]",
            "fa[a, b, c]",
            [
                {"xs" => "Sequence[a]", "ys" => "Sequence[b, c]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fa[b], c]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[b, fa[c]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fa[b], fa[c]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fa[b, c]]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[b, c]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[fa[b], c]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[b, fa[c]]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[fa[b], fa[c]]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[fa[b, c]]"},
                {"xs" => "Sequence[a, b]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[a, b]", "ys" => "Sequence[fa[c]]"},
                {"xs" => "Sequence[fa[a], b]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fa[a], b]", "ys" => "Sequence[fa[c]]"},
                {"xs" => "Sequence[a, fa[b]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[a, fa[b]]", "ys" => "Sequence[fa[c]]"},
                {"xs" => "Sequence[fa[a], fa[b]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fa[a], fa[b]]", "ys" => "Sequence[fa[c]]"},
                {"xs" => "Sequence[fa[a, b]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fa[a, b]]", "ys" => "Sequence[fa[c]]"},
            ]
        );

        // Multiple blank null sequence variables
        matcher_test!(
            multiple_blank_null_sequences,
            "fa[xs___, ys___]",
            "fa[a, b, c]",
            [
                {"xs" => "Sequence[]", "ys" => "Sequence[a, b, c]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[fa[a], b, c]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[a, fa[b], c]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[a, b, fa[c]]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[fa[a], fa[b], c]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[fa[a], b, fa[c]]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[a, fa[b], fa[c]]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[fa[a], fa[b], fa[c]]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[fa[a, b], c]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[fa[a, b], fa[c]]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[a, fa[b, c]]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[fa[a], fa[b, c]]"},
                {"xs" => "Sequence[]", "ys" => "Sequence[fa[a, b, c]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[b, c]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fa[b], c]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[b, fa[c]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fa[b], fa[c]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fa[b, c]]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[b, c]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[fa[b], c]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[b, fa[c]]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[fa[b], fa[c]]"},
                {"xs" => "Sequence[fa[a]]", "ys" => "Sequence[fa[b, c]]"},
                {"xs" => "Sequence[a, b]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[a, b]", "ys" => "Sequence[fa[c]]"},
                {"xs" => "Sequence[fa[a], b]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fa[a], b]", "ys" => "Sequence[fa[c]]"},
                {"xs" => "Sequence[a, fa[b]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[a, fa[b]]", "ys" => "Sequence[fa[c]]"},
                {"xs" => "Sequence[fa[a], fa[b]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fa[a], fa[b]]", "ys" => "Sequence[fa[c]]"},
                {"xs" => "Sequence[fa[a, b]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fa[a, b]]", "ys" => "Sequence[fa[c]]"},
                {"xs" => "Sequence[a, b, c]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[fa[a], b, c]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[a, fa[b], c]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[a, b, fa[c]]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[fa[a], fa[b], c]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[fa[a], b, fa[c]]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[a, fa[b], fa[c]]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[fa[a], fa[b], fa[c]]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[fa[a, b], c]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[fa[a, b], fa[c]]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[a, fa[b, c]]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[fa[a], fa[b, c]]", "ys" => "Sequence[]"},
                {"xs" => "Sequence[fa[a, b, c]]", "ys" => "Sequence[]"},
            ]
        );
    }

    mod associative_commutative {
        use super::*;

        // Unmatchable cases
        matcher_test!(mismatched_heads, "fac[a, b, c]", "g[a, b, c]", []);
        matcher_test!(extra_param, "fac[a, b]", "fac[a, b, c]", []);
        matcher_test!(blank_seq_empty, "fac[__]", "fac[]", []);

        // Multiple blank sequence variables
        matcher_test!(
            multiple_blank_sequences,
            "fac[xs__, ys__]",
            "fac[a, b, c]",
            [
                {"xs" => "Sequence[a]", "ys" => "Sequence[b, c]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fac[b], c]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[b, fac[c]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fac[b], fac[c]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fac[b, c]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[c, b]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fac[c], b]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[c, fac[b]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fac[c], fac[b]]"},
                {"xs" => "Sequence[a]", "ys" => "Sequence[fac[c, b]]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[b, c]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[fac[b], c]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[b, fac[c]]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[fac[b], fac[c]]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[fac[b, c]]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[c, b]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[fac[c], b]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[c, fac[b]]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[fac[c], fac[b]]"},
                {"xs" => "Sequence[fac[a]]", "ys" => "Sequence[fac[c, b]]"},
                {"xs" => "Sequence[a, b]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[a, b]", "ys" => "Sequence[fac[c]]"},
                {"xs" => "Sequence[fac[a], b]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fac[a], b]", "ys" => "Sequence[fac[c]]"},
                {"xs" => "Sequence[a, fac[b]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[a, fac[b]]", "ys" => "Sequence[fac[c]]"},
                {"xs" => "Sequence[fac[a], fac[b]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fac[a], fac[b]]", "ys" => "Sequence[fac[c]]"},
                {"xs" => "Sequence[fac[a, b]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fac[a, b]]", "ys" => "Sequence[fac[c]]"},
                {"xs" => "Sequence[b, a]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[b, a]", "ys" => "Sequence[fac[c]]"},
                {"xs" => "Sequence[fac[b], a]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fac[b], a]", "ys" => "Sequence[fac[c]]"},
                {"xs" => "Sequence[b, fac[a]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[b, fac[a]]", "ys" => "Sequence[fac[c]]"},
                {"xs" => "Sequence[fac[b], fac[a]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fac[b], fac[a]]", "ys" => "Sequence[fac[c]]"},
                {"xs" => "Sequence[fac[b, a]]", "ys" => "Sequence[c]"},
                {"xs" => "Sequence[fac[b, a]]", "ys" => "Sequence[fac[c]]"},

                // TODO: Where is `b` on `xs`?
            ]
        );
    }
}
