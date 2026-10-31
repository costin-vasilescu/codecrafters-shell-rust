use is_executable::IsExecutable;
use std::env;
use std::ffi::OsString;
use std::process::{Command, Output};

pub fn find_executable(command: &str) -> Option<OsString> {
    // load $PATH and create a split
    let path_var = env::var_os("PATH")?;
    let paths = env::split_paths(&path_var);

    // iterate directories from $PATH
    for path_dir in paths {
        let joined = path_dir.join(command);

        if joined.is_executable() {
            return Some(joined.into_os_string());
        }
    }
    None
}

pub fn handle_executable(command_args: &[String]) -> Option<Output> {
    // split executable command and arguments
    let (executable_cmd, args) = command_args.split_first()?;

    // run command with arguments
    Command::new(executable_cmd).args(args).output().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_executable() {
        let command_args: Vec<String> = vec!["ls".to_string()];
        handle_executable(&command_args);
    }
}
