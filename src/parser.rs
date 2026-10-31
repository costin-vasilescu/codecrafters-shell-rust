#[derive(PartialEq, Copy, Clone, Debug)]
enum ParserState {
    InsideSQ, // Single quotes
    InsideDQ, // Double quotes
    Normal,   // Unquoted
}

impl ParserState {
    fn is_qstate(&self) -> bool {
        *self == ParserState::InsideSQ || *self == ParserState::InsideDQ
    }

    fn state_change(&self, ch: char) -> Option<ParserState> {
        let new_state = match ch {
            '\'' => ParserState::InsideSQ,
            '"' => ParserState::InsideDQ,
            _ => ParserState::Normal,
        };

        match (*self, new_state) {
            (ParserState::Normal, ParserState::InsideSQ | ParserState::InsideDQ) => Some(new_state),
            (s1, s2) if s1.is_qstate() && s1 == s2 => Some(ParserState::Normal),
            (_, _) => None,
        }
    }
}

struct ShellParser {
    args: Vec<String>,
}

impl ShellParser {
    fn new() -> Self {
        ShellParser {
            args: Vec::from([String::new()]),
        }
    }

    fn is_last_empty(&mut self) -> bool {
        self.args.last().is_some_and(|t| t.is_empty())
    }

    fn push(&mut self, ch: char) {
        if let Some(last_token) = self.args.last_mut() {
            last_token.push(ch);
        }
    }

    fn push_new(&mut self) {
        self.args.push(String::new());
    }

    fn parse(&mut self, input: &str) {
        let mut input_chars = input.chars();
        let mut parser_state = ParserState::Normal;

        while let Some(ch) = input_chars.next() {
            // check for state transit
            if let Some(new_state) = parser_state.state_change(ch) {
                parser_state = new_state;

                continue;
            }

            match (ch, parser_state) {
                // backslash escapes only [", \, $, \n] in DQ state
                ('\\', ParserState::InsideDQ) => {
                    if let Some(next_ch) = input_chars.next() {
                        if !matches!(next_ch, '"' | '\\' | '$' | '\n') {
                            self.push(ch);
                        }
                        self.push(next_ch);
                    } else {
                        self.push(ch);
                    }
                }

                // backslash escapes everything in Normal state
                ('\\', ParserState::Normal) => {
                    if let Some(next_ch) = input_chars.next() {
                        self.push(next_ch);
                    } else {
                        self.push(ch);
                    }
                }

                // prevent repeating whitespaces in Normal state
                (ch, ParserState::Normal) if ch.is_whitespace() => {
                    if !self.is_last_empty() {
                        self.push_new();
                    }
                }

                // not a special character so just push
                (_, _) => {
                    self.push(ch);
                }
            }
        }

        // pop trailing empty string
        self.args.pop_if(|t| t.is_empty());
    }
}

pub fn parse_shell_input(input: &str) -> Vec<String> {
    let mut parser = ShellParser::new();
    parser.parse(input);

    parser.args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let test_cases = [
            (
                "'/tmp/rat/f   44' '/tmp/rat/f   91' '/tmp/rat/f   84'",
                vec!["/tmp/rat/f   44", "/tmp/rat/f   91", "/tmp/rat/f   84"],
            ),
            ("echo 'world shell'", vec!["echo", "world shell"]),
            ("echo test      shell", vec!["echo", "test", "shell"]),
            (
                "echo 'test''concatenated' \"another\"<<and'anotha'\"one\".txt",
                vec!["echo", "testconcatenated", "another<<andanothaone.txt"],
            ),
            ("test empty''quote", vec!["test", "emptyquote"]),
            (
                "/tmp/cow/'no slash 22' /tmp/cow/'one slash \\77' /tmp/cow/'two slashes \\60\\'",
                vec![
                    "/tmp/cow/no slash 22",
                    "/tmp/cow/one slash \\77",
                    "/tmp/cow/two slashes \\60\\",
                ],
            ),
        ];

        for (input, expected_args) in test_cases {
            let mut parser = ShellParser::new();
            parser.parse(input);

            assert_eq!(parser.args, expected_args, "Failed for input: {input}");
        }
    }
}
