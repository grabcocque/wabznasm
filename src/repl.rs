use crate::environment::{Environment, Value};
use crate::evaluator::Evaluator;
use crate::parser::parse_expression;
use color_eyre::eyre;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;

/// Run the interactive REPL with persistent environment.
pub fn run() -> Result<(), eyre::Report> {
    let mut rl: Editor<(), DefaultHistory> = Editor::new()?;
    let mut env = Environment::new();
    let evaluator = Evaluator::new();

    println!(
        "wabznasm REPL: enter expressions, assignments, or function definitions. Type 'exit' to quit"
    );
    println!("Examples: 1+2, f: {{x+1}}, add: {{[x;y] x+y}}, f[5], add[2;3]");

    loop {
        match rl.readline("wabz> ") {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(input);
                if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                    break;
                }

                // Parse and evaluate with persistent environment
                match parse_expression(input) {
                    Ok(tree) => match evaluator.eval_with_env(tree.root_node(), input, &mut env) {
                        Ok(Value::Integer(val)) => println!("= {}", val),
                        Ok(Value::Function { params, .. }) => {
                            if params.is_empty() {
                                println!("= {{expr}}");
                            } else {
                                println!("= {{[{}] expr}}", params.join(";"));
                            }
                        }
                        Err(e) => eprintln!("Error: {:?}", e),
                    },
                    Err(e) => eprintln!("Parse error: {:?}", e),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }
    Ok(())
}
