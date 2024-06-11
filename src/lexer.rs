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