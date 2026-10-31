use crate::executables::find_executable;
use std::env;

pub enum BuiltinResult {
    Output(String),
    Exit(i32),
    NoOutput,
    NotBuiltin,
}

pub fn handle_builtin(command_args: &[String]) -> BuiltinResult {
    let Some((builtin_cmd, args)) = command_args.split_first() else {
        return BuiltinResult::NotBuiltin;
    };

    match builtin_cmd.as_str() {
        "echo" => BuiltinResult::Output(args.join(" ")),
        "pwd" => BuiltinResult::Output(env::current_dir().unwrap().to_string_lossy().into_owned()),
        "exit" => match args.len() {
            1 => {
                let first_arg = args.first().unwrap();

                if let Ok(exit_code) = first_arg.parse() {
                    BuiltinResult::Exit(exit_code)
                } else {
                    BuiltinResult::Output(format!("{builtin_cmd}: {first_arg}: invalid integer"))
                }
            }
            0 => BuiltinResult::Exit(0),
            _ => BuiltinResult::Output(format!("{builtin_cmd}: too many arguments")),
        },
        "type" => {
            let Some(first_arg) = args.first() else {
                return BuiltinResult::NoOutput;
            };

            // 1. check if the command is a builtin
            if matches!(first_arg.as_str(), "echo" | "exit" | "type" | "pwd") {
                BuiltinResult::Output(format!("{first_arg} is a shell builtin"))
            }
            // 2. check for executable in each $PATH directory
            else if let Some(exec_path) = find_executable(first_arg) {
                BuiltinResult::Output(format!("{} is {}", first_arg, exec_path.to_string_lossy()))
            } else {
                BuiltinResult::Output(format!("{first_arg}: not found"))
            }
        }
        "cd" => {
            // args.first() returns None for bare cd
            // .filter() returns None for "cd ~"
            let cd_target = args.first().filter(|t| t.as_str() != "~");

            // map to $HOME if cd target is None
            let resolved_path = match cd_target {
                Some(s) => Some(s.to_owned()),
                None => env::var("HOME").ok(),
            };

            // if we still don't have a path, return error message
            let Some(resolved_path) = resolved_path else {
                return BuiltinResult::Output("Error: $HOME not set".to_string());
            };

            // try to change directory
            if env::set_current_dir(&resolved_path).is_err() {
                BuiltinResult::Output(format!("cd: {resolved_path}: No such file or directory"))
            } else {
                BuiltinResult::NoOutput
            }
        }
        _ => BuiltinResult::NotBuiltin,
    }
}
