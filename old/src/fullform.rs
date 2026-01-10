use crate::ast::Expr;
use crate::env::Env;
use crate::eval::eval;
use num_bigint::BigInt;
use std::fmt;
use std::fmt::Formatter;

pub fn assert_eval(env: &mut Env, input: &str, expected: &str) {
    let input_ff = parse_fullform(input).expect("parse input");
    let input_expr = from_fullform(input_ff, env);

    let expected_ff = parse_fullform(expected).expect("parse expected");

    let result_expr = eval(input_expr, env).expect("eval");
    let result_ff = into_fullform(result_expr, env);

    assert_eq!(result_ff, expected_ff);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FullForm {
    Symbol(String),
    Integer(BigInt),
    Apply(Box<FullForm>, Vec<FullForm>),
}

impl FullForm {
    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Symbol(name.into())
    }

    pub fn integer(value: impl Into<BigInt>) -> Self {
        Self::Integer(value.into())
    }

    pub fn apply(head: FullForm, args: impl Into<Vec<FullForm>>) -> Self {
        Self::Apply(Box::new(head), args.into())
    }
}

impl fmt::Display for FullForm {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FullForm::Symbol(symbol) => {
                write!(f, "{}", symbol)
            }

            FullForm::Integer(integer) => {
                write!(f, "{}", integer)
            }

            FullForm::Apply(head, args) => {
                write!(f, "{}[", head)?;

                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", arg)?;
                }

                write!(f, "]")
            }
        }
    }
}

pub fn into_fullform(expr: Expr, env: &Env) -> FullForm {
    match expr {
        Expr::Symbol(symbol_id) => FullForm::symbol(env.symbol_def(symbol_id).name()),
        Expr::Integer(integer) => FullForm::integer(integer),
        Expr::Apply(head, args) => FullForm::apply(
            into_fullform(*head, env),
            args.into_iter()
                .map(|arg| into_fullform(arg, env))
                .collect::<Vec<_>>(),
        ),
    }
}

pub fn from_fullform(expr: FullForm, env: &mut Env) -> Expr {
    match expr {
        FullForm::Symbol(symbol) => Expr::symbol(env.intern(&symbol)),
        FullForm::Integer(integer) => Expr::integer(integer),
        FullForm::Apply(head, args) => Expr::apply(
            from_fullform(*head, env),
            args.into_iter()
                .map(|arg| from_fullform(arg, env))
                .collect::<Vec<_>>(),
        ),
    }
}

pub fn parse_fullform(input: &str) -> Option<FullForm> {
    let mut parser = Parser { input, pos: 0 };

    let expr = parser.parse_expr()?;
    parser.skip_ws();

    if parser.pos == input.len() {
        Some(expr)
    } else {
        None
    }
}

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn parse_expr(&mut self) -> Option<FullForm> {
        self.skip_ws();

        let mut expr = if let Some(int) = self.parse_integer() {
            FullForm::integer(int)
        } else if let Some(symbol) = self.parse_symbol() {
            FullForm::symbol(symbol)
        } else {
            return None;
        };

        loop {
            self.skip_ws();
            if self.peek() == Some('[') {
                self.bump(); // '['
                let args = self.parse_args()?;
                expr = FullForm::apply(expr, args);
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_symbol(&mut self) -> Option<String> {
        self.skip_ws();

        let start = self.pos;

        match self.peek()? {
            c if c.is_ascii_alphabetic() || c == '$' => self.bump(),
            _ => return None,
        };

        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '$') {
            self.bump();
        }

        Some(self.input[start..self.pos].to_string())
    }

    fn parse_integer(&mut self) -> Option<BigInt> {
        self.skip_ws();

        let start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
        }

        if self.pos > start {
            Some(self.input[start..self.pos].parse().ok()?)
        } else {
            None
        }
    }

    fn parse_args(&mut self) -> Option<Vec<FullForm>> {
        let mut args = Vec::new();

        self.skip_ws();
        if self.peek() == Some(']') {
            self.bump();
            return Some(args);
        }

        loop {
            let expr = self.parse_expr()?;
            args.push(expr);

            self.skip_ws();
            match self.peek()? {
                ',' => {
                    self.bump();
                }
                ']' => {
                    self.bump();
                    break;
                }
                _ => return None,
            }
        }

        Some(args)
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.bump();
        }
    }
}
