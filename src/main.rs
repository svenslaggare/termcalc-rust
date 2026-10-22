use std::io::stdin;
use crate::calculator::Calculator;

mod parsing;
mod execution;
mod types;
mod model;
mod calculator;

fn main() {
    let mut calculator = Calculator::new();

    for line in stdin().lines() {
        match line {
            Ok(line) => {
                match calculator.execute(&line) {
                    Ok(mut result) => {
                        if !result.is_empty() {
                            if !result.ends_with("\n") {
                                result.push('\n');
                            }

                            print!("{}", result);
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
}
