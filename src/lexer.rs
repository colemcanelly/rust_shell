#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(String),
    Symbol(String),
    ControlOperator(String),
    Identifier(String),
    Wildcard(String),
    Str(String),
    Comment(String),
}

impl Token {
    pub fn inner(&self) -> &String {
        match self {
            Token::Literal(s) => s,
            Token::Symbol(s) => s,
            Token::ControlOperator(s) => s,
            Token::Identifier(s) => s,
            Token::Wildcard(s) => s,
            Token::Str(s) => s,
            Token::Comment(s) => s,
        }
    }

    pub fn inner_mut(&mut self) -> &mut String {
        match self {
            Token::Literal(s) => s,
            Token::Symbol(s) => s,
            Token::ControlOperator(s) => s,
            Token::Identifier(s) => s,
            Token::Wildcard(s) => s,
            Token::Str(s) => s,
            Token::Comment(s) => s,
        }
    }
}

mod state_machine {
    use super::Token;
    use std::mem;

    const VALID_PATH_CHARS: &[char] = &['_', '~', '/', '.'];
    const VARIABLE_CHARS: &[char] = &['_', '{', '}'];

    #[derive(Copy, Clone, Debug, Default, PartialEq)]
    pub enum LexerState {
        #[default]
        Start,
        InLiteral,
        InQuote(char),
        InSubstitution,
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
                    toks.push(Token::Symbol(ch.into()));
                    LexerState::InQuote(ch)
                }
                (LexerState::Start, '>' | '<') => {
                    cur_tok.push(ch);
                    LexerState::InOperator(ch)
                }
                (LexerState::Start, '|' | '=' | ';') => {
                    toks.push(Token::Symbol(ch.into()));
                    LexerState::Start
                }
                (LexerState::Start, '#') => {
                    cur_tok.push(ch);
                    LexerState::InComment
                }
                (LexerState::Start, '$') => {
                    toks.push(Token::Symbol('$'.into()));
                    LexerState::InSubstitution
                }
                (LexerState::Start | LexerState::InSubstitution | LexerState::InOperator(_), '(') => {
                    toks.push(Token::Symbol('('.into()));
                    LexerState::Start
                }
                (LexerState::Start | LexerState::InSubstitution | LexerState::InLiteral, ')') => {
                    toks.push(Token::Literal(mem::take(cur_tok)));
                    toks.push(Token::Symbol(')'.into()));
                    state_stack.pop().unwrap_or_default()
                }
                (LexerState::Start, _) => {
                    cur_tok.push(ch);
                    LexerState::InLiteral
                }
                (LexerState::InLiteral, c)
                    if c.is_alphanumeric() || VALID_PATH_CHARS.contains(&c) || c == '-' =>
                {
                    cur_tok.push(c);
                    LexerState::InLiteral
                }
                (LexerState::InLiteral, '*') => {
                    cur_tok.push(ch);
                    LexerState::InWildcard
                }
                (LexerState::InLiteral, _) => {
                    toks.push(match cur_tok.as_str() {
                        "if" | "then" | "else" | "fi" => Token::ControlOperator(mem::take(cur_tok)),
                        _ => Token::Literal(mem::take(cur_tok)),
                    });
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
                    toks.push(Token::Symbol(mem::take(cur_tok)));
                    LexerState::Start
                }
                (LexerState::InOperator(_), _) => {
                    toks.push(Token::Symbol(mem::take(cur_tok)));
                    LexerState::Start.tokenize_char(ch, cur_tok, toks, state_stack)
                }
                (LexerState::InSubstitution, c)
                    if c.is_alphanumeric() || VARIABLE_CHARS.contains(&c) =>
                {
                    cur_tok.push(c);
                    LexerState::InSubstitution
                }
                (LexerState::InSubstitution, _) => {
                    toks.push(Token::Identifier(mem::take(cur_tok)));
                    state_stack
                        .pop()
                        .unwrap_or_default()
                        .tokenize_char(ch, cur_tok, toks, state_stack)
                }
                (LexerState::InQuote(q), '$') => {
                    toks.push(Token::Str(mem::take(cur_tok)));
                    toks.push(Token::Symbol('$'.into()));
                    state_stack.push(LexerState::InQuote(q));
                    LexerState::InSubstitution
                }
                (LexerState::InQuote(q), c) if c != q => {
                    cur_tok.push(c);
                    LexerState::InQuote(q)
                }
                (LexerState::InQuote(_), _) => {
                    toks.push(Token::Str(mem::take(cur_tok)));
                    toks.push(Token::Symbol(ch.into()));
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
                LexerState::InLiteral => toks.push(Token::Literal(cur_tok)),
                // LexerState::InQuote(_) => toks.push(Token::Str(cur_tok)),
                LexerState::InSubstitution => toks.push(Token::Identifier(cur_tok)),
                LexerState::InWildcard => toks.push(Token::Wildcard(cur_tok)),
                LexerState::InOperator(_) => toks.push(Token::Symbol(cur_tok)),
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
            vec![Literal("~/bin/ansi_colors".into())]
        );
        // Programs with arguments
        assert_eq!(
            "ls -F --group-directories-first".tokenize(),
            vec![
                Literal("ls".into()),
                Literal("-F".into()),
                Literal("--group-directories-first".into())
            ]
        );
        assert_eq!(
            "xclip -selection c -o".tokenize(),
            vec![
                Literal("xclip".into()),
                Literal("-selection".into()),
                Literal("c".into()),
                Literal("-o".into())
            ]
        );
    }

    #[test]
    fn tokenize_trait() {
        assert_eq!(
            "~/bin/ansi_colors".tokenize(),
            vec![Literal("~/bin/ansi_colors".into())]
        );
        assert_eq!(
            String::from("~/bin/ansi_colors").tokenize(),
            vec![Literal("~/bin/ansi_colors".into())]
        );
    }

    #[test]
    fn quotes() {
        // String with no spaces inside
        assert_eq!(
            r#"grep ":Zone.Identifier""#.tokenize(),
            vec![
                Literal("grep".into()),
                Symbol('\"'.into()),
                Str(":Zone.Identifier".into()),
                Symbol('\"'.into())
            ]
        );

        assert_eq!(
            r#"echo "My name is Cole McAnelly""#.tokenize(),
            vec![
                Literal("echo".into()),
                Symbol('\"'.into()),
                Str("My name is Cole McAnelly".into()),
                Symbol('\"'.into())
            ]
        );

        // Single quotes with assignment operator
        assert_eq!(
            "alias colors='~/bin/ansi_colors'".tokenize(),
            vec![
                Literal("alias".into()),
                Literal("colors".into()),
                Symbol("=".into()),
                Symbol('\''.into()),
                Str("~/bin/ansi_colors".into()),
                Symbol('\''.into())
            ]
        );

        // Double quoted String with internal spaces, and assignment operator
        assert_eq!(
            r#"MY_VAR="this is the value of my variable""#.tokenize(),
            vec![
                Literal("MY_VAR".into()),
                Symbol("=".into()),
                Symbol('\"'.into()),
                Str("this is the value of my variable".into()),
                Symbol('\"'.into())
            ]
        );
    }

    #[test]
    fn pipes() {
        // Pipes with spaces in between
        assert_eq!(
            r#"history | grep git | xargs rm"#.tokenize(),
            vec![
                Literal("history".into()),
                Symbol("|".into()),
                Literal("grep".into()),
                Literal("git".into()),
                Symbol("|".into()),
                Literal("xargs".into()),
                Literal("rm".into())
            ]
        );
        assert_eq!(
            "ls ./src/*.rs | xargs basename -s .rs".tokenize(),
            vec![
                Literal("ls".into()),
                Wildcard("./src/*.rs".into()),
                Symbol("|".into()),
                Literal("xargs".into()),
                Literal("basename".into()),
                Literal("-s".into()),
                Literal(".rs".into())
            ]
        );

        // Pipes without spaces
        assert_eq!(
            r#"history|grep git|xargs rm"#.tokenize(),
            vec![
                Literal("history".into()),
                Symbol("|".into()),
                Literal("grep".into()),
                Literal("git".into()),
                Symbol("|".into()),
                Literal("xargs".into()),
                Literal("rm".into())
            ]
        );
    }

    #[test]
    fn io_redirections() {
        assert_eq!(
            r#"cat << EOF > file | wc -c | tr -d " " > file2"#.tokenize(),
            vec![
                Literal("cat".into()),
                Symbol("<<".into()),
                Literal("EOF".into()),
                Symbol(">".into()),
                Literal("file".into()),
                Symbol("|".into()),
                Literal("wc".into()),
                Literal("-c".into()),
                Symbol("|".into()),
                Literal("tr".into()),
                Literal("-d".into()),
                Symbol('\"'.into()),
                Str(" ".into()),
                Symbol('\"'.into()),
                Symbol(">".into()),
                Literal("file2".into())
            ]
        );

        assert_eq!(
            r#"echo "This is Cole McAnelly's file, and I am writing my name inside of it!!" >> my_file"#.tokenize(),
            vec![
                Literal("echo".into()),
                Symbol('\"'.into()),
                Str("This is Cole McAnelly's file, and I am writing my name inside of it!!".into()),
                Symbol('\"'.into()),
                Symbol(">>".into()),
                Literal("my_file".into())
            ]
        )
    }

    #[test]
    fn variables() {
        assert_eq!(
            "echo $VAR".tokenize(),
            vec![Literal("echo".into()), Symbol('$'.into()), Identifier("VAR".into())]
        );
        assert_eq!(
            r#"echo "this is $VAR right here""#.tokenize(),
            vec![
                Literal("echo".into()),
                Symbol('\"'.into()),
                Str("this is ".into()),
                Symbol('$'.into()),
                Identifier("VAR".into()),
                Str(" right here".into()),
                Symbol('\"'.into())
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
                Literal("echo".into()),
                Symbol('$'.into()),
                Symbol('('.into()),
                Literal("ls".into()),
                Literal("-a".into()),
                Symbol(')'.into())
            ]
        );
        assert_eq!(
            r#"echo -e "Here are the contents of the directory: [\n$(ls -a)\n]""#.tokenize(),
            vec![
                Literal("echo".into()),
                Literal("-e".into()),
                Symbol('\"'.into()),
                Str(r"Here are the contents of the directory: [\n".into()),
                Symbol('$'.into()),
                Symbol('('.into()),
                Literal("ls".into()),
                Literal("-a".into()),
                Symbol(')'.into()),
                Str(r"\n]".into()),
                Symbol('\"'.into()),
            ]
        );
        assert_eq!(
            r#"echo "$(ls -a)""#.tokenize(),
            vec![
                Literal("echo".into()),
                Symbol('\"'.into()),
                Str("".into()),
                Symbol('$'.into()),
                Symbol('('.into()),
                Literal("ls".into()),
                Literal("-a".into()),
                Symbol(')'.into()),
                Str("".into()),
                Symbol('\"'.into()),
            ]
        );
    }

    #[test]
    fn complex() {
        assert_eq!(
            "ls -l 'file name' | grep test $VAR # This is a comment".tokenize(),
            vec![
                Literal("ls".into()),
                Literal("-l".into()),
                Symbol('\''.into()),
                Str("file name".into()),
                Symbol('\''.into()),
                Symbol("|".into()),
                Literal("grep".into()),
                Literal("test".into()),
                Symbol('$'.into()),
                Identifier("VAR".into()),
                Comment("# This is a comment".into())
            ]
        );

        assert_eq!(
            r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize(),
            vec![
                Literal("find".into()),
                Literal(".".into()),
                Literal("-type".into()),
                Literal("f".into()),
                Symbol("|".into()),
                Literal("grep".into()),
                Symbol('\"'.into()),
                Str(":Zone.Identifier".into()),
                Symbol('\"'.into()),
                Symbol("|".into()),
                Literal("xargs".into()),
                Literal("rm".into())
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
