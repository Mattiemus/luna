use crate::env::Env;
use crate::eval::eval;
use crate::fullform::{from_fullform, into_fullform, parse_fullform};
use std::io;
use std::io::Write;

mod ast;
mod builtins;
mod env;
mod eval;
mod fullform;
mod pattern;
mod specificity;
mod symbol;

fn main() {
    let mut env = Env::new();

    // Set[f[0], 0]
    // Set[f[1], 1]
    // Set[f[Pattern[n, Blank[Integer]]], Plus[f[Subtract[n, 1], f[Subtract[n, 2]]]]]

    // Plus[x]            (* → x *)
    // Plus[Plus[x]]      (* → x *)

    // f[x_] := x
    // Set[f[Pattern[x, Blank[]]], x]
    // f[Plus[a]]         (* → a *)

    println!("Welcome to the LANG REPL");
    println!("Type `Quit` or press Ctrl-D to exit.\n");

    let stdin = io::stdin();

    loop {
        // Prompt
        print!("In> ");
        io::stdout().flush().unwrap();

        // Read line
        let mut input = String::new();
        let n = stdin.read_line(&mut input).unwrap();

        // EOF (Ctrl-D)
        if n == 0 {
            println!();
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "Quit" || input == "Exit" {
            break;
        }

        // Parse
        let fullform = match parse_fullform(input) {
            Some(ff) => ff,
            None => {
                eprintln!("Syntax error");
                continue;
            }
        };

        // Evaluate
        let expr = from_fullform(fullform, &mut env);

        match eval(expr, &mut env) {
            Ok(result) => {
                let ff = into_fullform(result, &env);
                println!("Out> {}", ff);
                println!();
            }
            Err(err) => {
                eprintln!("Evaluation error: {err:?}");
            }
        }
    }

    println!("Goodbye.");
}
