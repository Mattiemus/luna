use crate::Normal;
use crate::Symbol;
use crate::{BigFloat, BigInteger, Expr};
use rug::Complete;

// TODO: This should probably exist on the context?
pub const DEFAULT_REAL_PRECISION: u32 = 53;

// TODO: Make parsing less awful

#[macro_export]
macro_rules! parse {
    ($s:expr) => {
        parse($s).unwrap()
    };
}

pub fn parse(s: &str) -> Result<Expr, ()> {
    let mut c = Cursor::new(s);
    c.eat_whitespace();

    let expr = parse_expr(&mut c)?;
    c.eat_whitespace();

    if c.peek().is_some() {
        return Err(());
    }

    Ok(expr)
}

fn parse_expr(c: &mut Cursor) -> Result<Expr, ()> {
    let mut expr = parse_atom(c)?;

    loop {
        c.eat_whitespace();
        if c.peek() == Some('[') {
            expr = parse_application(c, expr)?;
        } else {
            break;
        }
    }

    Ok(expr)
}

fn parse_atom(c: &mut Cursor) -> Result<Expr, ()> {
    c.eat_whitespace();

    match c.peek() {
        Some('_') => parse_blank(c, None),
        Some(ch) if ch.is_ascii_digit() || ch == '-' => parse_number(c),
        Some('"') => parse_string(c),
        Some(ch) if is_symbol_start(ch) => parse_symbol_or_pattern(c),
        _ => Err(()),
    }
}

fn parse_number(c: &mut Cursor) -> Result<Expr, ()> {
    let mut s = String::new();
    let mut seen_dot = false;

    if c.peek() == Some('-') {
        s.push('-');
        c.bump();
    }

    while let Some(ch) = c.peek() {
        if ch.is_ascii_digit() {
            s.push(ch);
            c.bump();
        } else if ch == '.' && !seen_dot {
            seen_dot = true;
            s.push('.');
            c.bump();
        } else {
            break;
        }
    }

    if seen_dot {
        // real number
        BigFloat::parse(&s)
            .map(|f| Expr::from(BigFloat::with_val(DEFAULT_REAL_PRECISION, f)))
            .map_err(|_| ())
    } else {
        // integer
        BigInteger::parse(&s)
            .map(|i| Expr::from(i.complete()))
            .map_err(|_| ())
    }
}

fn parse_string(c: &mut Cursor) -> Result<Expr, ()> {
    // consume opening quote
    if c.bump() != Some('"') {
        return Err(());
    }

    let mut s = String::new();

    while let Some(ch) = c.bump() {
        match ch {
            '"' => {
                return Ok(Expr::from(s));
            }
            '\\' => {
                // simple escapes
                match c.bump() {
                    Some('"') => s.push('"'),
                    Some('\\') => s.push('\\'),
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some(other) => s.push(other),
                    None => return Err(()),
                }
            }
            _ => s.push(ch),
        }
    }

    Err(()) // unterminated string
}

fn parse_symbol_or_pattern(c: &mut Cursor) -> Result<Expr, ()> {
    let name = parse_symbol_name(c)?;

    // pattern suffix?
    if c.peek() == Some('_') {
        parse_blank(c, Some(name))
    } else {
        Ok(Expr::from(Symbol::new(&name)))
    }
}

fn is_symbol_start(c: char) -> bool {
    c.is_ascii_alphabetic()
}

fn parse_symbol_name(c: &mut Cursor) -> Result<String, ()> {
    let mut name = String::new();

    while let Some(ch) = c.peek() {
        if ch.is_ascii_alphanumeric() {
            name.push(ch);
            c.bump();
        } else {
            break;
        }
    }

    if name.is_empty() { Err(()) } else { Ok(name) }
}

fn parse_blank(c: &mut Cursor, name: Option<String>) -> Result<Expr, ()> {
    let mut count = 0;
    while c.peek() == Some('_') {
        c.bump();
        count += 1;
    }

    let mut pattern = match count {
        1 => Ok(Expr::from(Normal::new(Symbol::new("Blank"), vec![]))),
        2 => Ok(Expr::from(Normal::new(
            Symbol::new("BlankSequence"),
            vec![],
        ))),
        3 => Ok(Expr::from(Normal::new(
            Symbol::new("BlankNullSequence"),
            vec![],
        ))),
        _ => Err(()),
    }?;

    if let Some(name) = name {
        pattern = Expr::from(Normal::new(
            Symbol::new("Pattern"),
            vec![Expr::from(Symbol::new(&name)), pattern],
        ));
    }

    Ok(pattern)
}

fn parse_application(c: &mut Cursor, head: Expr) -> Result<Expr, ()> {
    // consume '['
    if c.bump() != Some('[') {
        return Err(());
    }

    let mut args = Vec::new();

    loop {
        c.eat_whitespace();

        if c.peek() == Some(']') {
            c.bump();
            break;
        }

        let arg = parse_expr(c)?;
        args.push(arg);

        c.eat_whitespace();

        match c.peek() {
            Some(',') => {
                c.bump();
            }
            Some(']') => {
                c.bump();
                break;
            }
            _ => return Err(()),
        }
    }

    Ok(Expr::from(Normal::new(head, args)))
}

#[derive(Clone)]
struct Cursor<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn eat_whitespace(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.bump();
        }
    }
}
