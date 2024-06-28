use std::mem::take;

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

pub trait Tokenize {
    fn tokenize(self) -> Vec<Token>;
}

impl<S> Tokenize for S
where
    S: AsRef<str>,
{
    fn tokenize(self) -> Vec<Token> {
        let mut fsm = self
            .as_ref()
            .trim()
            .chars()
            .fold(Lexer::default(), |fsm, ch| fsm.tokenize_char(ch));

        if !fsm.current.is_empty() {
            match fsm.state {
                LexerState::InLiteral => fsm.tokens.push(Token::Literal(fsm.current)),
                // LexerState::InQuote(_) => fsm.tokens.push(Token::Str(fsm.current)),
                LexerState::InSubstitution => fsm.tokens.push(Token::Identifier(fsm.current)),
                LexerState::InWildcard => fsm.tokens.push(Token::Wildcard(fsm.current)),
                LexerState::InOperator(_) => fsm.tokens.push(Token::Symbol(fsm.current)),
                LexerState::InComment => fsm.tokens.push(Token::Comment(fsm.current)),
                _ => (),
            }
        }

        fsm.tokens
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum LexerState {
    #[default]
    Start,
    InLiteral,
    InQuote(char),
    InSubstitution,
    InWildcard,
    InOperator(char),
    InComment,
}

#[derive(Debug, Default, PartialEq)]
struct Lexer {
    state: LexerState,
    stack: Vec<LexerState>,
    current: String,
    tokens: Vec<Token>,
}

impl Lexer {
    fn tokenize_char(mut self, ch: char) -> Self {
        match (self.state, ch) {
            (LexerState::Start, c) if c.is_whitespace() => (),
            (LexerState::Start, '\'' | '\"') => {
                self.tokens.push(Token::Symbol(ch.into()));
                self.state = LexerState::InQuote(ch);
            }
            (LexerState::Start, '>' | '<') => {
                self.current.push(ch);
                self.state = LexerState::InOperator(ch);
            }
            (LexerState::Start, '|' | '=' | ';') => self.tokens.push(Token::Symbol(ch.into())),
            (LexerState::Start, '#') => {
                self.current.push(ch);
                self.state = LexerState::InComment;
            }
            (LexerState::Start, '$') => {
                self.tokens.push(Token::Symbol('$'.into()));
                self.state = LexerState::InSubstitution;
            }
            (LexerState::Start | LexerState::InSubstitution | LexerState::InOperator(_), '(') => {
                self.tokens.push(Token::Symbol('('.into()));
                self.state = LexerState::Start;
            }
            (LexerState::Start | LexerState::InSubstitution | LexerState::InLiteral, ')') => {
                self.tokens.push(Token::Literal(take(&mut self.current)));
                self.tokens.push(Token::Symbol(')'.into()));
                self.state = self.stack.pop().unwrap_or_default();
            }
            (LexerState::Start, _) => {
                self.current.push(ch);
                self.state = LexerState::InLiteral;
            }
            (LexerState::InLiteral, c) if c.is_path_char() => self.current.push(c),
            (LexerState::InLiteral, '*') => {
                self.current.push(ch);
                self.state = LexerState::InWildcard;
            }
            (LexerState::InLiteral, _) => {
                self.tokens.push(match self.current.as_str() {
                    "if" | "then" | "else" | "fi" => {
                        Token::ControlOperator(take(&mut self.current))
                    }
                    _ => Token::Literal(take(&mut self.current)),
                });
                self.state = LexerState::Start;
                self = self.tokenize_char(ch);
            }
            (LexerState::InWildcard, c) if c.is_path_char() => self.current.push(c),
            (LexerState::InWildcard, _) => {
                self.tokens.push(Token::Wildcard(take(&mut self.current)));
                self.state = LexerState::Start;
                self = self.tokenize_char(ch);
            }
            (LexerState::InOperator(io_dir), c) if c == io_dir => {
                self.current.push(c);
                self.tokens.push(Token::Symbol(take(&mut self.current)));
                self.state = LexerState::Start;
            }
            (LexerState::InOperator(_), _) => {
                self.tokens.push(Token::Symbol(take(&mut self.current)));
                self.state = LexerState::Start;
                self = self.tokenize_char(ch);
            }
            (LexerState::InSubstitution, c) if c.is_variable_char() => self.current.push(c),
            (LexerState::InSubstitution, _) => {
                self.tokens.push(Token::Identifier(take(&mut self.current)));
                self.state = self.stack.pop().unwrap_or_default();
                self = self.tokenize_char(ch);
            }
            (LexerState::InQuote(_), '$') => {
                self.tokens.push(Token::Str(take(&mut self.current)));
                self.tokens.push(Token::Symbol('$'.into()));
                self.stack.push(self.state);
                self.state = LexerState::InSubstitution;
            }
            (LexerState::InQuote(q), c) if c != q => self.current.push(c),
            (LexerState::InQuote(_), _) => {
                self.tokens.push(Token::Str(take(&mut self.current)));
                self.tokens.push(Token::Symbol(ch.into()));
                self.state = LexerState::Start;
            }
            (LexerState::InComment, '\n') => {
                self.tokens.push(Token::Comment(take(&mut self.current)));
                self.state = LexerState::Start;
            }
            (LexerState::InComment, c) => self.current.push(c),
        }
        self
    }
}

trait Valid {
    fn is_variable_char(self) -> bool;
    fn is_path_char(self) -> bool;
}

impl Valid for char {
    fn is_variable_char(self) -> bool {
        self.is_alphanumeric() || ['_', '{', '}'].contains(&self)
    }

    fn is_path_char(self) -> bool {
        self.is_alphanumeric() || ['_', '~', '/', '.', '-'].contains(&self)
    }
}
