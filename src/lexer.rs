#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ControlFlow {
    IF,
    THEN,
    ELSE,
    ENDIF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    CmdArg(String),
    Operator(String),
    ControlOperator(ControlFlow),
    Identifier(String),
    DollarSign,
    GroupOpen,
    GroupClose,
    Wildcard(String),
    QuoteBegin(char),
    QuoteContents(String),
    QuoteEnd(char),
    Comment(String),
}

mod state_machine {
    use super::{ControlFlow, Token};
    use std::mem;

    const VALID_PATH_CHARS: &[char] = &['_', '~', '/', '.'];
    const VARIABLE_CHARS: &[char] = &['_', '{', '}'];

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

    impl LexerState {
        pub fn tokenize_char(
            self,
            ch: char,
            cur_tok: &mut String,
            toks: &mut Vec<Token>,
            state_stack: &mut Vec<LexerState>,
        ) -> LexerState {
            match (self, ch) {
                (LexerState::Start, c) if c.is_whitespace() => LexerState::Start,
                (LexerState::Start, '\'' | '\"') => {
                    toks.push(Token::QuoteBegin(ch));
                    LexerState::InQuote(ch)
                }
                (LexerState::Start, '>' | '<') => {
                    cur_tok.push(ch);
                    LexerState::InOperator(ch)
                }
                (LexerState::Start, '|' | '=' | ';') => {
                    toks.push(Token::Operator(ch.into()));
                    LexerState::Start
                }
                (LexerState::Start, '#') => {
                    cur_tok.push(ch);
                    LexerState::InComment
                }
                (LexerState::Start, '$') => {
                    toks.push(Token::DollarSign);
                    LexerState::InVariable
                }
                (LexerState::Start | LexerState::InVariable | LexerState::InOperator(_), '(') => {
                    toks.push(Token::GroupOpen);
                    LexerState::Start
                }
                (LexerState::Start | LexerState::InVariable | LexerState::InCmdArg, ')') => {
                    toks.push(Token::CmdArg(mem::take(cur_tok)));
                    toks.push(Token::GroupClose);
                    state_stack.pop().or(Some(LexerState::Start)).unwrap()
                }
                (LexerState::Start, _) => {
                    cur_tok.push(ch);
                    LexerState::InCmdArg
                }
                (LexerState::InCmdArg, c)
                    if c.is_alphanumeric() || VALID_PATH_CHARS.contains(&c) || c == '-' =>
                {
                    cur_tok.push(c);
                    LexerState::InCmdArg
                }
                (LexerState::InCmdArg, '*') => {
                    cur_tok.push(ch);
                    LexerState::InWildcard
                }
                (LexerState::InCmdArg, _) => {
                    toks.push(match cur_tok.as_str() {
                        "if" => Token::ControlOperator(ControlFlow::IF),
                        "then" => Token::ControlOperator(ControlFlow::THEN),
                        "else" => Token::ControlOperator(ControlFlow::ELSE),
                        "fi" => Token::ControlOperator(ControlFlow::ENDIF),
                        _ => Token::CmdArg(mem::take(cur_tok)),
                    });
                    cur_tok.clear();
                    LexerState::Start.tokenize_char(ch, cur_tok, toks, state_stack)
                }
                (LexerState::InWildcard, c)
                    if c.is_alphanumeric() || VALID_PATH_CHARS.contains(&c) =>
                {
                    cur_tok.push(c);
                    LexerState::InWildcard
                }
                (LexerState::InWildcard, _) => {
                    toks.push(Token::Wildcard(mem::take(cur_tok)));
                    LexerState::Start.tokenize_char(ch, cur_tok, toks, state_stack)
                }
                (LexerState::InOperator(io_dir), c) if c == io_dir => {
                    cur_tok.push(c);
                    toks.push(Token::Operator(mem::take(cur_tok)));
                    LexerState::Start
                }
                (LexerState::InOperator(_), _) => {
                    toks.push(Token::Operator(mem::take(cur_tok)));
                    LexerState::Start.tokenize_char(ch, cur_tok, toks, state_stack)
                }
                (LexerState::InVariable, c)
                    if c.is_alphanumeric() || VARIABLE_CHARS.contains(&c) =>
                {
                    cur_tok.push(c);
                    LexerState::InVariable
                }
                (LexerState::InVariable, _) => {
                    toks.push(Token::Identifier(mem::take(cur_tok)));
                    state_stack
                        .pop()
                        .or(Some(LexerState::Start))
                        .unwrap()
                        .tokenize_char(ch, cur_tok, toks, state_stack)
                }
                (LexerState::InQuote(q), '$') => {
                    toks.push(Token::QuoteContents(mem::take(cur_tok)));
                    toks.push(Token::DollarSign);
                    state_stack.push(LexerState::InQuote(q));
                    LexerState::InVariable
                }
                (LexerState::InQuote(q), c) if c != q => {
                    cur_tok.push(c);
                    LexerState::InQuote(q)
                }
                (LexerState::InQuote(_), _) => {
                    toks.push(Token::QuoteContents(mem::take(cur_tok)));
                    toks.push(Token::QuoteEnd(ch));
                    LexerState::Start
                }
                (LexerState::InComment, '\n') => {
                    toks.push(Token::Comment(mem::take(cur_tok)));
                    LexerState::Start
                }
                (LexerState::InComment, c) => {
                    cur_tok.push(c);
                    LexerState::InComment
                }
            }
        }
    }
}

pub trait Tokenize {
    fn tokenize(self) -> Vec<Token>;
}

impl<S> Tokenize for S
where
    S: AsRef<str>,
{
    fn tokenize(self) -> Vec<Token> {
        use crate::lexer::state_machine::LexerState;

        let mut toks = vec![];
        let mut prev_states = vec![];
        let mut cur_tok = String::new();

        let fsm_state = self
            .as_ref()
            .trim()
            .chars()
            .fold(LexerState::Start, |fsm, ch| {
                fsm.tokenize_char(ch, &mut cur_tok, &mut toks, &mut prev_states)
            });

        if !cur_tok.is_empty() {
            match fsm_state {
                LexerState::InCmdArg => toks.push(Token::CmdArg(cur_tok)),
                // LexerState::InQuote(_) => toks.push(Token::Quote(cur_tok)),
                LexerState::InVariable => toks.push(Token::Identifier(cur_tok)),
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
            vec![
                CmdArg("grep".into()),
                QuoteBegin('\"'),
                QuoteContents(":Zone.Identifier".into()),
                QuoteEnd('\"')
            ]
        );

        assert_eq!(
            r#"echo "My name is Cole McAnelly""#.tokenize(),
            vec![
                CmdArg("echo".into()),
                QuoteBegin('\"'),
                QuoteContents("My name is Cole McAnelly".into()),
                QuoteEnd('\"')
            ]
        );

        // Single quotes with assignment operator
        assert_eq!(
            "alias colors='~/bin/ansi_colors'".tokenize(),
            vec![
                CmdArg("alias".into()),
                CmdArg("colors".into()),
                Operator("=".into()),
                QuoteBegin('\''),
                QuoteContents("~/bin/ansi_colors".into()),
                QuoteEnd('\'')
            ]
        );

        // Double quoted String with internal spaces, and assignment operator
        assert_eq!(
            r#"MY_VAR="this is the value of my variable""#.tokenize(),
            vec![
                CmdArg("MY_VAR".into()),
                Operator("=".into()),
                QuoteBegin('\"'),
                QuoteContents("this is the value of my variable".into()),
                QuoteEnd('\"')
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
                QuoteBegin('\"'),
                QuoteContents(" ".into()),
                QuoteEnd('\"'),
                Operator(">".into()),
                CmdArg("file2".into())
            ]
        );

        assert_eq!(
            r#"echo "This is Cole McAnelly's file, and I am writing my name inside of it!!" >> my_file"#.tokenize(),
            vec![
                CmdArg("echo".into()),
                QuoteBegin('\"'),
                QuoteContents("This is Cole McAnelly's file, and I am writing my name inside of it!!".into()),
                QuoteEnd('\"'),
                Operator(">>".into()),
                CmdArg("my_file".into())
            ]
        )
    }

    #[test]
    fn variables() {
        assert_eq!(
            "echo $VAR".tokenize(),
            vec![CmdArg("echo".into()), DollarSign, Identifier("VAR".into())]
        );
        assert_eq!(
            r#"echo "this is $VAR right here""#.tokenize(),
            vec![
                CmdArg("echo".into()),
                QuoteBegin('\"'),
                QuoteContents("this is ".into()),
                DollarSign,
                Identifier("VAR".into()),
                QuoteContents(" right here".into()),
                QuoteEnd('\"')
            ]
        );
    }
    #[test]
    fn parenthesis() {
        // Pipes with spaces in between
        // todo!("SUBPROCESS TOKENIZING");
        assert_eq!(
            "echo $(ls -a)".tokenize(),
            vec![
                CmdArg("echo".into()),
                DollarSign,
                GroupOpen,
                CmdArg("ls".into()),
                CmdArg("-a".into()),
                GroupClose
            ]
        );
        assert_eq!(
            r#"echo -e "Here are the contents of the directory: [\n$(ls -a)\n]""#.tokenize(),
            vec![
                CmdArg("echo".into()),
                CmdArg("-e".into()),
                QuoteBegin('\"'),
                QuoteContents(r"Here are the contents of the directory: [\n".into()),
                DollarSign,
                GroupOpen,
                CmdArg("ls".into()),
                CmdArg("-a".into()),
                GroupClose,
                QuoteContents(r"\n]".into()),
                QuoteEnd('\"'),
            ]
        );
    }

    #[test]
    fn complex() {
        // assert_eq!(
        //     r#"echo "$(ls -a)""#.tokenize(),
        //     vec!["echo", "\"", "$", "(", "ls", "-a", ")", "\""]
        // );
        assert_eq!(
            "ls -l 'file name' | grep test $VAR # This is a comment".tokenize(),
            vec![
                CmdArg("ls".into()),
                CmdArg("-l".into()),
                QuoteBegin('\''),
                QuoteContents("file name".into()),
                QuoteEnd('\''),
                Operator("|".into()),
                CmdArg("grep".into()),
                CmdArg("test".into()),
                DollarSign,
                Identifier("VAR".into()),
                Comment("# This is a comment".into())
            ]
        );

        assert_eq!(
            r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize(),
            vec![
                CmdArg("find".into()),
                CmdArg(".".into()),
                CmdArg("-type".into()),
                CmdArg("f".into()),
                Operator("|".into()),
                CmdArg("grep".into()),
                QuoteBegin('\"'),
                QuoteContents(":Zone.Identifier".into()),
                QuoteEnd('\"'),
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
