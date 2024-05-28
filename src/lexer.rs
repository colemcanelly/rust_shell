#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    CmdArg(String),
    Operator(String),
    ControlOperator(String),
    Variable(String),
    Wildcard(String),
    Quote(String),
    // QuoteFragment(String),
    Comment(String),
}

mod state_machine {
    use super::Token;


    const COMMAND_CHARS: &[char] = &['_', '~', '/', '.'];
    const VARIABLE_CHARS: &[char] = &['_', '{', '}'];
    const CONTROL_FLOW: &[&str] = &["if", "then", "else", "fi"];
    
    #[derive(Copy, Clone, Debug, Default, PartialEq)]
    pub enum LexerState {
        #[default]
        Start,
        InCmdArg,
        InQuote(char),
        InVariable,
        InWildcard,
        InOperator(char),
        InComment,
    }

    pub fn tokenize_char(
        cur_state: LexerState,
        c: char,
        cur_tok: &mut String,
        toks: &mut Vec<Token>,
    ) -> LexerState {
        match (cur_state, c) {
            (LexerState::Start, c) if c.is_whitespace() => LexerState::Start,
            (LexerState::Start, '\'' | '\"') => {
                cur_tok.push(c);
                LexerState::InQuote(c)
            }
            (LexerState::Start, '|' | '>' | '<' | '=' | ';') => {
                if c == '<' || c == '>' {
                    cur_tok.push(c);
                    LexerState::InOperator(c)
                } else {
                    toks.push(Token::Operator(c.to_string()));
                    LexerState::Start
                }
            }
            (LexerState::Start, '#') => {
                cur_tok.push(c);
                LexerState::InComment
            }
            (LexerState::Start, '$') => {
                cur_tok.push(c);
                LexerState::InVariable
            }
            (LexerState::Start, _) => {
                cur_tok.push(c);
                LexerState::InCmdArg
            }
            (LexerState::InCmdArg, c)
                if c.is_alphanumeric() || COMMAND_CHARS.contains(&c) || c == '-' =>
            {
                cur_tok.push(c);
                LexerState::InCmdArg
            }
            (LexerState::InCmdArg, '*') => {
                cur_tok.push(c);
                LexerState::InWildcard
            }
            (LexerState::InCmdArg, _) => {
                if CONTROL_FLOW.contains(&cur_tok.as_str()) {
                    toks.push(Token::ControlOperator(cur_tok.clone()));
                } else {
                    toks.push(Token::CmdArg(cur_tok.clone()));
                }
                cur_tok.clear();
                tokenize_char(LexerState::Start, c, cur_tok, toks)
            }
            (LexerState::InWildcard, c) if c.is_alphanumeric() || COMMAND_CHARS.contains(&c) => {
                cur_tok.push(c);
                LexerState::InWildcard
            }
            (LexerState::InWildcard, _) => {
                toks.push(Token::Wildcard(cur_tok.clone()));
                cur_tok.clear();
                tokenize_char(LexerState::Start, c, cur_tok, toks)
            }
            (LexerState::InOperator(io_dir), c) if c == io_dir => {
                cur_tok.push(c);
                toks.push(Token::Operator(cur_tok.to_string()));
                cur_tok.clear();
                LexerState::Start
            }
            (LexerState::InOperator(_), _) => {
                toks.push(Token::Operator(cur_tok.to_string()));
                cur_tok.clear();
                tokenize_char(LexerState::Start, c, cur_tok, toks)
            }
            (LexerState::InQuote(quote_char), c) if c != quote_char => {
                cur_tok.push(c);
                LexerState::InQuote(quote_char)
            }
            (LexerState::InQuote(_), _) => {
                cur_tok.push(c);
                toks.push(Token::Quote(cur_tok.clone()));
                cur_tok.clear();
                LexerState::Start
            }
            (LexerState::InVariable, c) if c.is_alphanumeric() || VARIABLE_CHARS.contains(&c) => {
                cur_tok.push(c);
                LexerState::InVariable
            }
            (LexerState::InVariable, _) => {
                toks.push(Token::Variable(cur_tok.clone()));
                cur_tok.clear();
                tokenize_char(LexerState::Start, c, cur_tok, toks)
            }
            (LexerState::InComment, '\n') => {
                toks.push(Token::Comment(cur_tok.clone()));
                cur_tok.clear();
                LexerState::Start
            }
            (LexerState::InComment, c) => {
                cur_tok.push(c);
                LexerState::InComment
            }
        }
    }
}

pub trait Tokenize {
    fn tokenize(self) -> Vec<Token>;
}

impl<S: AsRef<str>> Tokenize for S {
    fn tokenize(self) -> Vec<Token> {
        use crate::lexer::state_machine::{tokenize_char, LexerState};

        let mut toks = vec![];
        let mut cur_tok = String::new();

        let state = self
            .as_ref()
            .trim()
            .chars()
            .fold(LexerState::Start, |state, ch| {
                tokenize_char(state, ch, &mut cur_tok, &mut toks)
            });

        if !cur_tok.is_empty() {
            match state {
                LexerState::InCmdArg => toks.push(Token::CmdArg(cur_tok)),
                LexerState::InQuote(_) => toks.push(Token::Quote(cur_tok)),
                LexerState::InVariable => toks.push(Token::Variable(cur_tok)),
                LexerState::InWildcard => toks.push(Token::Wildcard(cur_tok)),
                LexerState::InOperator(_) => toks.push(Token::Operator(cur_tok)),
                LexerState::InComment => toks.push(Token::Comment(cur_tok)),
                _ => (),
            }
        }

        toks
    }
}

#[cfg(test)]
mod test {
    use super::Token::*;
    use super::*;

    #[test]
    fn command_arguments() {
        // Programs without arguments
        assert_eq!(
            "~/bin/ansi_colors".tokenize(),
            vec![CmdArg("~/bin/ansi_colors".into())]
        );
        // Programs with arguments
        assert_eq!(
            "ls -F --group-directories-first".tokenize(),
            vec![
                CmdArg("ls".into()),
                CmdArg("-F".into()),
                CmdArg("--group-directories-first".into())
            ]
        );
        assert_eq!(
            "xclip -selection c -o".tokenize(),
            vec![
                CmdArg("xclip".into()),
                CmdArg("-selection".into()),
                CmdArg("c".into()),
                CmdArg("-o".into())
            ]
        );
    }

    #[test]
    fn tokenize_trait() {
        assert_eq!(
            "~/bin/ansi_colors".tokenize(),
            vec![CmdArg("~/bin/ansi_colors".into())]
        );
        assert_eq!(
            String::from("~/bin/ansi_colors").tokenize(),
            vec![CmdArg("~/bin/ansi_colors".into())]
        );
    }

    #[test]
    fn quotes() {
        // String with no spaces inside
        assert_eq!(
            r#"grep ":Zone.Identifier""#.tokenize(),
            vec![CmdArg("grep".into()), Quote(r#"":Zone.Identifier""#.into())]
        );

        assert_eq!(
            r#"echo "My name is Cole McAnelly""#.tokenize(),
            vec![
                CmdArg("echo".into()),
                Quote(r#""My name is Cole McAnelly""#.into())
            ]
        );

        // Single quotes with assignment operator
        assert_eq!(
            "alias colors='~/bin/ansi_colors'".tokenize(),
            vec![
                CmdArg("alias".into()),
                CmdArg("colors".into()),
                Operator("=".into()),
                Quote("'~/bin/ansi_colors'".into())
            ]
        );

        // Double quoted String with internal spaces, and assignment operator
        assert_eq!(
            r#"MY_VAR="this is the value of my variable""#.tokenize(),
            vec![
                CmdArg("MY_VAR".into()),
                Operator("=".into()),
                Quote(r#""this is the value of my variable""#.into())
            ]
        );
    }

    #[test]
    fn pipes() {
        // Pipes with spaces in between
        assert_eq!(
            r#"history | grep git | xargs rm"#.tokenize(),
            vec![
                CmdArg("history".into()),
                Operator("|".into()),
                CmdArg("grep".into()),
                CmdArg("git".into()),
                Operator("|".into()),
                CmdArg("xargs".into()),
                CmdArg("rm".into())
            ]
        );
        assert_eq!(
            "ls ./src/*.rs | xargs basename -s .rs".tokenize(),
            vec![
                CmdArg("ls".into()),
                Wildcard("./src/*.rs".into()),
                Operator("|".into()),
                CmdArg("xargs".into()),
                CmdArg("basename".into()),
                CmdArg("-s".into()),
                CmdArg(".rs".into())
            ]
        );

        // Pipes without spaces
        assert_eq!(
            r#"history|grep git|xargs rm"#.tokenize(),
            vec![
                CmdArg("history".into()),
                Operator("|".into()),
                CmdArg("grep".into()),
                CmdArg("git".into()),
                Operator("|".into()),
                CmdArg("xargs".into()),
                CmdArg("rm".into())
            ]
        );
    }

    #[test]
    fn quotes_pipes_and_variables() {
        assert_eq!(
            "ls -l 'file name' | grep test $VAR # This is a comment".tokenize(),
            vec![
                CmdArg("ls".into()),
                CmdArg("-l".into()),
                Quote("'file name'".into()),
                Operator("|".into()),
                CmdArg("grep".into()),
                CmdArg("test".into()),
                Variable("$VAR".into()),
                Comment("# This is a comment".into())
            ]
        )
    }

    #[test]
    fn io_redirections() {
        assert_eq!(
            r#"cat << EOF > file | wc -c | tr -d " " > file2"#.tokenize(),
            vec![
                CmdArg("cat".into()),
                Operator("<<".into()),
                CmdArg("EOF".into()),
                Operator(">".into()),
                CmdArg("file".into()),
                Operator("|".into()),
                CmdArg("wc".into()),
                CmdArg("-c".into()),
                Operator("|".into()),
                CmdArg("tr".into()),
                CmdArg("-d".into()),
                Quote(r#"" ""#.into()),
                Operator(">".into()),
                CmdArg("file2".into())
            ]
        );

        assert_eq!(
            r#"echo "This is Cole McAnelly's file, and I am writing my name inside of it!!" >> my_file"#.tokenize(),
            vec![
                CmdArg("echo".into()),
                Quote(r#""This is Cole McAnelly's file, and I am writing my name inside of it!!""#.into()),
                Operator(">>".into()),
                CmdArg("my_file".into())
            ]
        )
    }

    #[test]
    fn parenthesis() {
        // Pipes with spaces in between
        todo!("SUBPROCESS TOKENIZING");
        // assert_eq!(
        //     "echo $(ls -a)".tokenize(),
        //     vec!["echo", "$", "(", "ls", "-a", ")"]
        // );
    }

    #[test]
    fn multiple() {
        // assert_eq!(
        //     r#"echo "$(ls -a)""#.tokenize(),
        //     vec!["echo", "\"", "$", "(", "ls", "-a", ")", "\""]
        // );
        assert_eq!(
            r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize(),
            vec![
                CmdArg("find".into()),
                CmdArg(".".into()),
                CmdArg("-type".into()),
                CmdArg("f".into()),
                Operator("|".into()),
                CmdArg("grep".into()),
                Quote(r#"":Zone.Identifier""#.into()),
                Operator("|".into()),
                CmdArg("xargs".into()),
                CmdArg("rm".into())
            ]
        );
        // assert_eq!(
        //     "stow $@ 2> >(grep -v 'BUG in find_stowed_path? Absolute/relative mismatch' 1>&2)".tokenize(),
        //     vec![
        //         "stow",
        //         "$@",
        //         "2>",
        //         ">",
        //         "(",
        //         "grep",
        //         "-v",
        //         "'",
        //         "BUG",
        //         "in",
        //         "find_stowed_path?",
        //         "Absolute/relative",
        //         "mismatch",
        //         "'",
        //         "1>&2",
        //         ")"
        //     ]
        // );
    }
}
