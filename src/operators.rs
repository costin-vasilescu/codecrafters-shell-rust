use crate::parser::parse_shell_input;
use crate::utils::write_file;

const OPCHARS: [u8; 2] = *b">|";

#[derive(Debug, PartialEq)]
enum OperatorKind {
    Append,      // >> | 1>>
    Redirect,    // > | 1>
    AppendErr,   // 2>>
    RedirectErr, // 2>
}

#[derive(Debug, PartialEq)]
pub struct ShellOperator {
    kind: OperatorKind,
    args: Vec<String>,
}

impl ShellOperator {
    fn is_write_operator(&self) -> bool {
        let write_types = [
            OperatorKind::Append,
            OperatorKind::AppendErr,
            OperatorKind::Redirect,
            OperatorKind::RedirectErr,
        ];

        write_types.contains(&self.kind)
    }

    pub fn handle_operator(&self, stdout: &[u8], stderr: &[u8]) {
        if self.is_write_operator() {
            let write_path = self.args.join(" ");

            let append_flag = matches!(&self.kind, OperatorKind::Append | OperatorKind::AppendErr);
            let error_flag = matches!(
                &self.kind,
                OperatorKind::AppendErr | OperatorKind::RedirectErr
            );

            if error_flag {
                if let Err(err) = write_file(&write_path, stderr, append_flag) {
                    eprintln!("shell: {write_path}: {err}");
                }
                print!("{}", String::from_utf8_lossy(stdout));
            } else {
                if let Err(err) = write_file(&write_path, stdout, append_flag) {
                    eprintln!("shell: {write_path}: {err}");
                }
                eprint!("{}", String::from_utf8_lossy(stderr));
            }
        }
    }
}

pub fn split_at_operator(input: &str) -> (&str, Option<ShellOperator>) {
    let Some(i) = input.bytes().position(|ch| OPCHARS.contains(&ch)) else {
        return (input, None);
    };

    // explicit stream operators like " 1>" start with a byte offset
    let std_prefix_len = match &input[..i] {
        cmd if cmd.ends_with(" 1") || cmd.ends_with(" 2") => 2,
        _ => 0,
    };
    let op_start = i - std_prefix_len;
    let rest = &input[op_start..];

    // longest token first: " 1>>" before " 1>", ">>" before ">"
    let op_len = match rest {
        r if r.starts_with(" 1>>") || r.starts_with(" 2>>") => 4,
        r if r.starts_with(" 1>") || r.starts_with(" 2>") => 3,
        r if r.starts_with(">>") => 2,
        r if r.starts_with(">") => 1,
        _ => unreachable!(),
    };

    let (command, rest) = input.split_at(op_start);
    let (operator, operand) = rest.split_at(op_len);

    let kind = match operator {
        ">>" | " 1>>" => OperatorKind::Append,
        ">" | " 1>" => OperatorKind::Redirect,
        " 2>>" => OperatorKind::AppendErr,
        " 2>" => OperatorKind::RedirectErr,
        _ => unreachable!(),
    };

    let shell_operator = ShellOperator {
        kind,
        args: parse_shell_input(operand),
    };

    (command, Some(shell_operator))
}

#[cfg(test)]
mod test {
    use super::*;

    fn new_op(kind: OperatorKind, args: &[&str]) -> Option<ShellOperator> {
        Some(ShellOperator {
            kind,
            args: args.iter().map(|arg| arg.to_string()).collect(),
        })
    }

    #[test]
    fn test_operator_split() {
        assert_eq!(
            split_at_operator("test>test2"),
            ("test", new_op(OperatorKind::Redirect, &["test2"]))
        );
        assert_eq!(
            split_at_operator("test>>test2"),
            ("test", new_op(OperatorKind::Append, &["test2"]))
        );
        assert_eq!(
            split_at_operator("echo hi > out.txt"),
            ("echo hi ", new_op(OperatorKind::Redirect, &["out.txt"]))
        );
        assert_eq!(
            split_at_operator("echo > 'a b' c"),
            ("echo ", new_op(OperatorKind::Redirect, &["a b", "c"]))
        );
    }
}
