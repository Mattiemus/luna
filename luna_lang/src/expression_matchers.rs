use crate::Expr;
use crate::symbol::Symbol;

/// Matches an expression of the form `Sequence[___]`.
pub fn is_sequence(expr: &Expr) -> bool {
    expr.is_normal_head(&Symbol::new("Sequence"))
}

/// Matches an expression of any the following forms:
///
/// - `Blank[]`
/// - `Blank[h]`
pub fn is_blank(expr: &Expr) -> bool {
    if let Some(v) = expr.try_normal_head(&Symbol::new("Blank")) {
        return matches!(v.elements().len(), 0 | 1);
    }

    false
}

/// Matches an expression of any the following forms:
///
/// - `Blank[]`
/// - `Blank[h]`
pub fn try_blank(expr: &Expr) -> Option<Option<&Expr>> {
    if let Some(v) = expr.try_normal_head(&Symbol::new("Blank")) {
        if matches!(v.elements().len(), 0 | 1) {
            return Some(v.elements().get(0));
        }
    }

    None
}

/// Matches an expression of any the following forms:
///
/// - `Pattern[sym, Blank[]]`
/// - `Pattern[sym, Blank[h]]`
pub fn try_blank_pattern(expr: &Expr) -> Option<(&Symbol, Option<&Expr>)> {
    if let Some(v) = expr.try_normal_head(&Symbol::new("Pattern")) {
        if let [s, p] = v.elements() {
            if let Some(sym) = s.try_symbol() {
                return try_blank(p).map(|h| (sym, h));
            }
        }
    }

    None
}

/// Parses an expression of the following forms:
///
/// - `Blank[]`
/// - `Blank[h]`
/// - `Pattern[sym, Blank[]]`
/// - `Pattern[sym, Blank[h]]`
pub fn parse_blank_pattern(expr: &Expr) -> Option<(Option<&Symbol>, Option<&Expr>)> {
    if let Some(h) = try_blank(&expr) {
        return Some((None, h));
    }

    if let Some((variable, h)) = try_blank_pattern(&expr) {
        return Some((Some(variable), h));
    }

    None
}

/// Matches an expression of any the following forms:
///
/// - `BlankSequence[]`
/// - `BlankSequence[h]`
/// - `BlankNullSequence[]`
/// - `BlankNullSequence[h]`
pub fn is_any_blank_sequence(expr: &Expr) -> bool {
    if let Some(v) = expr.try_normal() {
        if !matches!(v.elements().len(), 1 | 2) {
            return false;
        }

        return v.has_head(&Symbol::new("BlankSequence"))
            || v.has_head(&Symbol::new("BlankNullSequence"));
    }

    false
}

/// Matches an expression of any the following forms:
///
/// - `Pattern[sym, BlankSequence[]]`
/// - `Pattern[sym, BlankSequence[h]]`
/// - `Pattern[sym, BlankNullSequence[]]`
/// - `Pattern[sym, BlankNullSequence[h]]`
pub fn is_any_sequence_pattern(expr: &Expr) -> bool {
    if let Some(v) = expr.try_normal_head(&Symbol::new("Pattern")) {
        if let [s, p] = v.elements() {
            if let Some(_) = s.try_symbol() {
                return is_any_blank_sequence(p);
            }
        }
    }

    false
}
