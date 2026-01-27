use crate::Expr;
use crate::Symbol;

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
/// - `BlankSequence[]`
/// - `BlankSequence[h]`
pub fn is_blank_sequence(expr: &Expr) -> bool {
    if let Some(v) = expr.try_normal_head(&Symbol::new("BlankSequence")) {
        return matches!(v.elements().len(), 0 | 1);
    }

    false
}

/// Matches an expression of any the following forms:
///
/// - `BlankNullSequence[]`
/// - `BlankNullSequence[h]`
pub fn is_blank_null_sequence(expr: &Expr) -> bool {
    if let Some(v) = expr.try_normal_head(&Symbol::new("BlankNullSequence")) {
        return matches!(v.elements().len(), 0 | 1);
    }

    false
}

/// Matches an expression of any the following forms:
///
/// - `BlankSequence[]`
/// - `BlankSequence[h]`
/// - `BlankNullSequence[]`
/// - `BlankNullSequence[h]`
pub fn is_any_blank_sequence(expr: &Expr) -> bool {
    is_blank_sequence(expr) || is_blank_null_sequence(expr)
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

/// Matches an expression of any the following forms:
///
/// - `BlankSequence[]`
/// - `BlankSequence[h]`
/// - `BlankNullSequence[]`
/// - `BlankNullSequence[h]`
/// - `Pattern[sym, BlankSequence[]]`
/// - `Pattern[sym, BlankSequence[h]]`
/// - `Pattern[sym, BlankNullSequence[]]`
/// - `Pattern[sym, BlankNullSequence[h]]`
pub fn is_any_sequence_variable(expr: &Expr) -> bool {
    is_any_blank_sequence(expr) || is_any_sequence_pattern(expr)
}

/// Matches an expression of the form `Sequence[___]`.
pub fn try_sequence(expr: &Expr) -> Option<&[Expr]> {
    expr.try_normal_head(&Symbol::new("Sequence"))
        .map(|seq| seq.elements())
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
/// - `BlankSequence[]`
/// - `BlankSequence[h]`
pub fn try_blank_sequence(expr: &Expr) -> Option<Option<&Expr>> {
    if let Some(v) = expr.try_normal_head(&Symbol::new("BlankSequence")) {
        if matches!(v.elements().len(), 0 | 1) {
            return Some(v.elements().get(0));
        }
    }

    None
}

/// Matches an expression of any the following forms:
///
/// - `BlankNullSequence[]`
/// - `BlankNullSequence[h]`
pub fn try_blank_null_sequence(expr: &Expr) -> Option<Option<&Expr>> {
    if let Some(v) = expr.try_normal_head(&Symbol::new("BlankNullSequence")) {
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

/// Matches an expression of any the following forms:
///
/// - `Pattern[sym, BlankSequence[]]`
/// - `Pattern[sym, BlankSequence[h]]`
pub fn try_blank_sequence_pattern(expr: &Expr) -> Option<(&Symbol, Option<&Expr>)> {
    if let Some(v) = expr.try_normal_head(&Symbol::new("Pattern")) {
        if let [s, p] = v.elements() {
            if let Some(sym) = s.try_symbol() {
                return try_blank_sequence(p).map(|h| (sym, h));
            }
        }
    }

    None
}

/// Matches an expression of any the following forms:
///
/// - `Pattern[sym, BlankNullSequence[]]`
/// - `Pattern[sym, BlankNullSequence[h]]`
pub fn try_blank_null_sequence_pattern(expr: &Expr) -> Option<(&Symbol, Option<&Expr>)> {
    if let Some(v) = expr.try_normal_head(&Symbol::new("Pattern")) {
        if let [s, p] = v.elements() {
            if let Some(sym) = s.try_symbol() {
                return try_blank_null_sequence(p).map(|h| (sym, h));
            }
        }
    }

    None
}

/// Parses an expression of any the following forms:
///
/// - `Blank[]`
/// - `Blank[h]`
/// - `Pattern[sym, Blank[]]`
/// - `Pattern[sym, Blank[h]]`
pub fn parse_individual_variable(expr: &Expr) -> Option<(Option<&Symbol>, Option<&Expr>)> {
    if let Some(h) = try_blank(&expr) {
        return Some((None, h));
    }

    if let Some((variable, h)) = try_blank_pattern(&expr) {
        return Some((Some(variable), h));
    }

    None
}

/// Parses an expression of any the following forms:
///
/// - `BlankSequence[]`
/// - `BlankSequence[h]`
/// - `BlankNullSequence[]`
/// - `BlankNullSequence[h]`
/// - `Pattern[sym, BlankSequence[]]`
/// - `Pattern[sym, BlankSequence[h]]`
/// - `Pattern[sym, BlankNullSequence[]]`
/// - `Pattern[sym, BlankNullSequence[h]]`
pub fn parse_any_sequence_variable(expr: &Expr) -> Option<(bool, Option<&Symbol>, Option<&Expr>)> {
    if let Some(h) = try_blank_sequence(&expr) {
        return Some((false, None, h));
    }

    if let Some(h) = try_blank_null_sequence(&expr) {
        return Some((true, None, h));
    }

    if let Some((variable, h)) = try_blank_sequence_pattern(&expr) {
        return Some((false, Some(variable), h));
    }

    if let Some((variable, h)) = try_blank_null_sequence_pattern(&expr) {
        return Some((true, Some(variable), h));
    }

    None
}

pub fn extract_condition(expr: &Expr) -> (&Expr, Option<&Expr>) {
    if let Some(normal) = expr.try_normal_head(&Symbol::new("Condition")) {
        if normal.len() == 2 {
            return (&normal.elements()[0], Some(&normal.elements()[1]));
        }
    }

    (expr, None)
}
