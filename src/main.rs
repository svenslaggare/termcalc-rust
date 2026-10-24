use rustyline::{DefaultEditor};
use crate::calculator::{CalculationError, Calculator};

mod parsing;
mod execution;
mod types;
mod model;
mod calculator;
mod number_helpers;
mod plot;

fn main() {
    let mut calculator = Calculator::new();

    let mut line_editor = DefaultEditor::new().unwrap();
    while let Ok(mut line) = line_editor.readline("> ") {
        if line.ends_with('\n') {
            line.pop();
        }

        line_editor.add_history_entry(line.clone()).unwrap();
        match calculator.execute(&line) {
            Ok(mut result) => {
                if !result.is_empty() {
                    if !result.ends_with("\n") {
                        result.push('\n');
                    }

                    print!("{}", result);
                }
            }
            Err(CalculationError::Exit) => {
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
}
