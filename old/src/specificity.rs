use crate::ast::{Expr, is_blank, is_blank_null_sequence, is_blank_sequence, is_pattern};
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
enum SpecificityClass {
    Atom,                   // x
    Apply,                  // f[...]
    BlankTyped,             // Blank[h]
    Blank,                  // Blank[]
    BlankSequenceTyped,     // BlankSequence[h]
    BlankSequence,          // BlankSequence[]
    BlankNullSequenceTyped, // BlankNullSequence[h]
    BlankNullSequence,      // BlankNullSequence[]
}

impl SpecificityClass {
    fn for_expr(expr: &Expr) -> Self {
        // Pattern[x, p] → classify(p)
        if let Some((_, subpat)) = is_pattern(expr) {
            return Self::for_expr(subpat);
        }

        if let Some(c) = is_blank(expr) {
            return match c {
                Some(_) => SpecificityClass::BlankTyped,
                None => SpecificityClass::Blank,
            };
        }

        if let Some(c) = is_blank_sequence(expr) {
            return match c {
                Some(_) => SpecificityClass::BlankSequenceTyped,
                None => SpecificityClass::BlankSequence,
            };
        }

        if let Some(c) = is_blank_null_sequence(expr) {
            return match c {
                Some(_) => SpecificityClass::BlankNullSequenceTyped,
                None => SpecificityClass::BlankNullSequence,
            };
        }

        match expr {
            Expr::Symbol(_) => SpecificityClass::Atom,
            Expr::Integer(_) => SpecificityClass::Atom,
            Expr::Apply(_, _) => SpecificityClass::Apply,
        }
    }
}

pub fn compare_specificity(lhs: &Expr, rhs: &Expr) -> Ordering {
    // Compare top-level specificity class
    let c1 = SpecificityClass::for_expr(lhs);
    let c2 = SpecificityClass::for_expr(rhs);
    match c1.cmp(&c2) {
        Ordering::Equal => {}
        ord => return ord,
    }

    // Prefer more structure (more nodes)
    let n1 = node_count(lhs);
    let n2 = node_count(rhs);
    match n2.cmp(&n1) {
        Ordering::Equal => {}
        ord => return ord,
    }

    // Prefer deeper patterns
    let d1 = depth(lhs);
    let d2 = depth(rhs);
    match d2.cmp(&d1) {
        Ordering::Equal => {}
        ord => return ord,
    }

    // Stable fallback
    Ordering::Equal
}

fn depth(expr: &Expr) -> usize {
    match expr {
        Expr::Symbol(_) => 1,
        Expr::Integer(_) => 1,
        Expr::Apply(head, args) => {
            1 + std::cmp::max(depth(&**head), args.iter().map(depth).max().unwrap_or(0))
        }
    }
}

fn node_count(expr: &Expr) -> usize {
    match expr {
        Expr::Symbol(_) => 1,
        Expr::Integer(_) => 1,
        Expr::Apply(head, args) => {
            1 + node_count(&**head) + args.iter().map(node_count).sum::<usize>()
        }
    }
}
