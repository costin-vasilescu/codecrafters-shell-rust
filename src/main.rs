use std::io::{self, Write};

use crate::builtins::{BuiltinResult, handle_builtin};
use crate::executables::handle_executable;
use crate::operators::split_at_operator;
use crate::parser::parse_shell_input;

mod builtins;
mod executables;
mod operators;
mod parser;
mod utils;

fn main() {
    // REPL
    loop {
        // display prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // read user input
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // split input into command and optional operator struct
        let (cmd, op_opt) = split_at_operator(input);

        // if command is a builtin handle it and print output
        let shell_cmd = parse_shell_input(cmd);

        match handle_builtin(&shell_cmd) {
            BuiltinResult::Output(output) => {
                match op_opt {
                    Some(shell_op) => {
                        shell_op.handle_operator(format!("{output}\n").as_bytes(), &[])
                    }
                    None => println!("{output}"),
                }
                continue;
            }
            BuiltinResult::Exit(exit_code) => {
                std::process::exit(exit_code);
            }
            BuiltinResult::NoOutput => {
                continue;
            }
            BuiltinResult::NotBuiltin => (),
        }

        // try to handle command as executable
        // if unsuccessful print command not found
        let Some(shell_out) = handle_executable(&shell_cmd) else {
            println!("{cmd}: command not found");
            continue;
        };

        // try to handle operator
        // if no operator print stdout and stderr normally
        if let Some(shell_op) = op_opt {
            shell_op.handle_operator(&shell_out.stdout, &shell_out.stderr);
        } else {
            print!("{}", String::from_utf8_lossy(&shell_out.stdout));
            eprint!("{}", String::from_utf8_lossy(&shell_out.stderr));
        }
    }
}
