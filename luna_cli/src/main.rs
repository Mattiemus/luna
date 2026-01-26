use luna_lang::{Context, evaluate, parse};
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RLResult};

struct Session {
    context: Context,
}

impl Session {
    fn new() -> Self {
        Self {
            context: Context::new_global_context(),
        }
    }

    fn process_input(&mut self, input: &str) -> Result<(), String> {
        let result = parse(input);

        match result {
            Ok(expr) => {
                let result = evaluate(expr, &mut self.context);
                println!();
                println!("{}", result);
                println!();
                Ok(())
            }

            _ => Err("Failed to parse.\n".to_string()),
        }
    }
}

fn main() -> RLResult<()> {
    println!("Luna! A language for scientific computing");
    println!("v0.1.0");
    println!();

    let mut session = Session::new();

    // TODO: Add helper
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline(":> ");

        match readline {
            Ok(line) => {
                _ = rl.add_history_entry(line.as_str());

                match line.as_str() {
                    "end" | "exit" => break,
                    _ => {
                        // No-op
                    }
                }

                match session.process_input(line.as_str()) {
                    Ok(()) => {}
                    Err(msg) => {
                        println!("{}", msg);
                    }
                }
            }

            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }

            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }

            Err(err) => {
                println!("Error: {:?}\n", err);
            }
        }
    }

    Ok(())
}
