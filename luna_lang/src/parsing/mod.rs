use crate::Symbol;
use crate::{BigFloat, Normal};
use crate::{BigInteger, Expr};

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{escaped_transform, tag, take_until, take_while1},
    character::complete::{char, digit1, multispace0, one_of},
    combinator::{cut, map, opt, peek, recognize},
    error::ParseError,
    multi::{many0, many1, separated_list0},
    number::complete::recognize_float,
    sequence::{delimited, pair, preceded, terminated},
};
use nom::combinator::eof;
use rug::ops::CompleteRound;

// TODO: This should probably exist on the context?
pub const DEFAULT_REAL_PRECISION: u32 = 53;

#[macro_export]
macro_rules! parse {
    ($s:expr) => {
        crate::parse_str($s).unwrap()
    };
}

pub fn parse_str(expr: &str) -> Result<Expr, String> {
    match parse_root(expr) {
        Err(error) => Err(format!("Error while parsing: {}", error)),
        Ok((_, result)) => Ok(result),
    }
}

fn parse_root(i: &str) -> IResult<&str, Expr> {
    let (i, _) = many0(parse_comment).parse(i)?;
    let (i, _) = multispace0(i)?;
    let (i, expr) = signed_expr.parse(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = eof(i)?;

    Ok((i, expr))
}

fn parse_comment(i: &str) -> IResult<&str, &str> {
    delimited(tag("(*"), take_until("*)"), tag("*)")).parse(i)
}

fn signed_expr(i: &str) -> IResult<&str, Expr> {
    let (i, first_head) = expr(i)?;
    let (i, mut list_infixes) = many0(pair(parse_infix_operator, expr)).parse(i)?;

    list_infixes.insert(0, ((Symbol::new(""), u8::MAX), first_head));

    while list_infixes.len() > 1 {
        let mut max_priority = 0;
        let mut max_priority_position = 1;

        for (x, ((_, priority), _)) in list_infixes.iter().skip(1).enumerate() {
            if max_priority < *priority {
                max_priority = *priority;
                max_priority_position = x + 1;
            }
        }

        let ((infix_operator, _), post_infix) = list_infixes.remove(max_priority_position);
        let ((previous_infix_operator, previous_priority), new_child) =
            list_infixes.remove(max_priority_position - 1);

        let new_head = Expr::from(Normal::new(infix_operator, vec![new_child, post_infix]));

        list_infixes.insert(
            max_priority_position - 1,
            ((previous_infix_operator, previous_priority), new_head),
        );
    }

    let ((_, _), final_head) = &list_infixes[0];

    Ok((i, final_head.clone()))
}

fn expr(i: &str) -> IResult<&str, Expr> {
    let (i, _) = multispace0(i)?;
    let (i, _) = many0(parse_comment).parse(i)?;
    let (i, _) = multispace0(i)?;

    let (i, mut new_head) = alt((
        parse_slot,
        parse_array,
        parse_parenthesized,
        parse_part,
        parse_function,
        parse_num,
        parse_pattern,
        parse_symbol,
        parse_string,
        parse_association,
    ))
    .parse(i)?;

    let (i, _) = multispace0(i)?;
    let (i, children_from_at_sign) = opt(preceded(char('@'), expr)).parse(i)?;

    if let Some(child) = children_from_at_sign {
        return Ok((i, Expr::from(Normal::new(new_head, vec![child]))));
    }

    let (i, part) = opt(delimited(
        preceded(multispace0, tag("[[")),
        expr,
        preceded(multispace0, tag("]]")),
    ))
    .parse(i)?;

    if let Some(p) = part {
        return Ok((
            i,
            Expr::from(Normal::new(Symbol::new("Part"), vec![new_head, p])),
        ));
    }

    // Handle postfix operators: !, !!, '
    // These can be chained, e.g., 5!! or f''
    let (i, postfix_ops) = many0(parse_single_postfix_op).parse(i)?;
    for op in postfix_ops {
        match op {
            "!" => {
                new_head = Expr::from(Normal::new(Symbol::new("Factorial"), vec![new_head]));
            }
            "!!" => {
                new_head = Expr::from(Normal::new(Symbol::new("Factorial2"), vec![new_head]));
            }
            "'" => {
                new_head = Expr::from(Normal::new(
                    Symbol::new("Lookup"),
                    vec![
                        Expr::from(Normal::new(
                            Symbol::new("Derivative"),
                            vec![Expr::from(BigInteger::ONE.clone())],
                        )),
                        new_head,
                    ],
                ));
            }
            _ => {}
        }
    }

    let (i, _) = multispace0(i)?;
    let (i, _) = many0(parse_comment).parse(i)?;
    let (i, _) = multispace0(i)?;

    let (i, lambda) = opt(char('&')).parse(i)?;

    if lambda.is_some() {
        let (i, _) = multispace0(i)?;

        Ok((
            i,
            Expr::from(Normal::new(Symbol::new("Function"), vec![new_head])),
        ))
    } else {
        Ok((i, new_head))
    }
}

fn parse_slot(i: &str) -> IResult<&str, Expr> {
    let (i, _) = tag("#")(i)?;
    let (i, opt_slot_num) = opt(digit1).parse(i)?;

    let slot_num = match opt_slot_num {
        None => 1,
        Some(num) => num.parse::<i32>().unwrap(),
    };

    Ok((
        i,
        Expr::from(Normal::new(
            Symbol::new("Slot"),
            vec![Expr::from(BigInteger::from(slot_num))],
        )),
    ))
}

fn parse_num(i: &str) -> IResult<&str, Expr> {
    let (i, potential_sign) = opt(tag("-")).parse(i)?;
    let (_, potential_num) = peek(recognize_float).parse(i)?;

    let sign = if potential_sign.is_some() { -1 } else { 1 };

    if potential_num.contains('.') {
        map(recognize_float, |r| {
            Expr::from(BigFloat::parse(r).unwrap().complete(DEFAULT_REAL_PRECISION))
        })
        .parse(i)
    } else {
        map(digit1, |r| {
            Expr::from(BigInteger::from_str_radix(r, 10).unwrap() * sign)
        })
        .parse(i)
    }
}

fn unescape_string(i: &str) -> IResult<&str, &str> {
    let (i, str) = alt((tag("\\"), tag("\""))).parse(i)?;
    match str {
        "\\" => Ok((i, "\\")),
        "\"" => Ok((i, "\"")),
        _ => Ok((i, "")),
    }
}

fn parse_pattern(i: &str) -> IResult<&str, Expr> {
    let (i, potential_name) = opt(recognize(parse_symbol)).parse(i)?;
    let (i, underscores) = take_while1(|c| c == '_').parse(i)?;
    let (i, potential_head) = opt(recognize(parse_symbol)).parse(i)?;

    let pattern_head = match potential_head {
        None => vec![],
        Some(head) => vec![Expr::from(Symbol::new(head))],
    };

    let pattern = match underscores.len() {
        1 => Expr::from(Normal::new(Symbol::new("Blank"), pattern_head)),
        2 => Expr::from(Normal::new(Symbol::new("BlankSequence"), pattern_head)),
        3 => Expr::from(Normal::new(Symbol::new("BlankNullSequence"), pattern_head)),
        _ => {
            // More than 3 underscores is invalid, but we'll treat as ZeroOrMore
            Expr::from(Normal::new(Symbol::new("BlankNullSequence"), pattern_head))
        }
    };

    Ok(match potential_name {
        None => (i, pattern),
        Some(name) => (
            i,
            Expr::from(Normal::new(
                Symbol::new("Pattern"),
                vec![Expr::from(Symbol::new(name)), pattern],
            )),
        ),
    })
}

fn parse_string(i: &str) -> IResult<&str, Expr> {
    let (i, potential_empty_string) = opt(tag("\"\"")).parse(i)?;

    if potential_empty_string.is_some() {
        return Ok((i, Expr::from("".to_owned())));
    }

    let (i, content) = preceded(
        char('\"'),
        cut(terminated(
            escaped_transform(
                take_while1(|c| c != '\"' && c != '\\'),
                '\\',
                unescape_string,
            ),
            char('\"'),
        )),
    )
    .parse(i)?;

    Ok((i, Expr::from(content)))
}

fn parse_symbol<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Expr, E> {
    map(
        many1(one_of(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789$",
        )),
        |symbol_str| Expr::from(Symbol::new(&symbol_str.into_iter().collect::<String>())),
    )
    .parse(i)
}

fn parse_infix_operator(i: &str) -> IResult<&str, (Symbol, u8)> {
    let (i, _) = multispace0(i)?;

    // IMPORTANT: Operators are ordered longest-first within each starting character
    // to ensure proper matching (e.g., @@@ before @@, === before ==)
    let (i, (op, priority)) = alt((
        // Three-character operators (must come first)
        alt((
            tag("@@@").map(|_| (Symbol::new("MapApply"), 120)),
            tag("=!=").map(|_| (Symbol::new("UnsameQ"), 20)),
            tag("===").map(|_| (Symbol::new("SameQ"), 20)),
            tag("//.").map(|_| (Symbol::new("ReplaceRepeated"), 13)),
        )),
        // Two-character operators
        alt((
            tag("@@").map(|_| (Symbol::new("Apply"), 120)),
            tag("/@").map(|_| (Symbol::new("Map"), 120)),
            tag("/.").map(|_| (Symbol::new("ReplaceAll"), 13)),
            tag("//").map(|_| (Symbol::new("PostfixApplication"), 10)),
            tag("<>").map(|_| (Symbol::new("StringJoin"), 90)),
            tag("<=").map(|_| (Symbol::new("LessEqual"), 26)),
            tag(":>").map(|_| (Symbol::new("RuleDelayed"), 14)),
            tag(":=").map(|_| (Symbol::new("SetDelayed"), 11)),
            tag(">=").map(|_| (Symbol::new("GreaterEqual"), 25)),
            tag("->").map(|_| (Symbol::new("Rule"), 15)),
            tag("==").map(|_| (Symbol::new("Equal"), 21)),
            tag("!=").map(|_| (Symbol::new("Unequal"), 21)),
            tag(";;").map(|_| (Symbol::new("Span"), 80)),
            tag("&&").map(|_| (Symbol::new("And"), 4)),
            tag("||").map(|_| (Symbol::new("Or"), 3)),
        )),
        // Single-character operators (must come last)
        alt((
            tag("<").map(|_| (Symbol::new("Less"), 26)),
            tag(">").map(|_| (Symbol::new("Greater"), 25)),
            tag("=").map(|_| (Symbol::new("Set"), 12)),
            tag("+").map(|_| (Symbol::new("Plus"), 60)),
            tag("-").map(|_| (Symbol::new("Subtract"), 50)),
            tag("*").map(|_| (Symbol::new("Times"), 100)),
            tag("/").map(|_| (Symbol::new("Divide"), 105)),
            tag("^").map(|_| (Symbol::new("Power"), 101)),
            tag(";").map(|_| (Symbol::new("CompoundExpression"), 2)),
        )),
    ))
    .parse(i)?;

    let (i, _) = multispace0(i)?;

    Ok((i, (op, priority)))
}

fn parse_part(i: &str) -> IResult<&str, Expr> {
    let (i, expr) = parse_symbol(i)?;
    let (i, mut exprs) = preceded(
        tag("[["),
        cut(terminated(
            separated_list0(preceded(multispace0, char(',')), signed_expr),
            preceded(multispace0, tag("]]")),
        )),
    )
    .parse(i)?;

    let mut elems = vec![expr];
    elems.append(&mut exprs);

    Ok((i, Expr::from(Normal::new(Symbol::new("Part"), elems))))
}

fn parse_function(i: &str) -> IResult<&str, Expr> {
    let (i, expr) = alt((
        parse_slot,
        parse_array,
        parse_parenthesized,
        parse_part,
        parse_num,
        parse_pattern,
        parse_symbol,
        parse_string,
        parse_association,
    ))
    .parse(i)?;

    let (i, exprs) = many1(preceded(
        char('['),
        cut(terminated(
            separated_list0(preceded(multispace0, char(',')), signed_expr),
            preceded(multispace0, char(']')),
        )),
    ))
    .parse(i)?;

    let mut new_head = expr;
    for elems in exprs {
        new_head = Expr::from(Normal::new(new_head, elems));
    }

    Ok((i, new_head))
}

fn parse_association(i: &str) -> IResult<&str, Expr> {
    let (i, exprs) = preceded(
        tag("<|"),
        cut(terminated(
            separated_list0(preceded(multispace0, char(',')), signed_expr),
            preceded(multispace0, tag("|>")),
        )),
    )
    .parse(i)?;

    Ok((
        i,
        Expr::from(Normal::new(Symbol::new("Association"), exprs)),
    ))
}
fn parse_array(i: &str) -> IResult<&str, Expr> {
    let (i, exprs) = preceded(
        char('{'),
        cut(terminated(
            separated_list0(preceded(multispace0, char(',')), signed_expr),
            preceded(multispace0, char('}')),
        )),
    )
    .parse(i)?;

    Ok((i, Expr::from(Normal::new(Symbol::new("List"), exprs))))
}

fn parse_parenthesized(i: &str) -> IResult<&str, Expr> {
    let (i, _) = char('(')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, expr) = signed_expr(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = char(')')(i)?;

    Ok((i, expr))
}

fn parse_single_postfix_op(i: &str) -> IResult<&str, &str> {
    alt((
        tag("!!"),
        terminated(tag("!"), peek(nom::combinator::not(char('=')))),
        tag("'"),
    ))
    .parse(i)
}
