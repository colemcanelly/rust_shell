use core::str;
use core::cell::Cell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'t> {
    Invalid,

    // 1 wide
    // Newline,
    #[doc = ";"] Semicolon,        // ;
    #[doc = "|"] Pipe,             // |
    #[doc = "`"] BackTick,         // `
    #[doc = "*"] Asterisk,         // *
    #[doc = "$"] Dollar,           // $
    #[doc = "("] LeftParenthesis,  // (
    #[doc = ")"] RightParenthesis, // )
    #[doc = "{"] LeftBrace,        // {
    #[doc = "}"] RightBrace,       // }
    #[doc = "<"] LessThan,         // <
    #[doc = ">"] MoreThan,         // >
    #[doc = "="] Assign,
    #[doc = "="] Bang,             // !

    // 2 wide
    #[doc = "=="] Equals,    // ==
    #[doc = "!="] NotEquals, // !=
    #[doc = ">>"] Append,    // >>
    #[doc = "$("] Shell,     // $(

    // Reserved Keywords
    If,
    Then,
    Else,
    Fi,

    // Variadic
    Literal(&'t str),
    Path(&'t str),
    Flag(&'t str),      // <-flag> | <--flag>
    Ident(&'t str),     // $IDENT
    DoubleStr(&'t str), // "<string>"
    SingleStr(&'t str), // '<string>'
    FormatStr(&'t str), // `<STR>${ARG}<STR>$(SHELL)<STR>`
}

impl Token<'_> {
    #[inline]
    pub const fn len(self) -> usize {
        use Token::*;
        match self {
            Invalid => 0,
            Semicolon | Pipe | BackTick | Asterisk | Dollar | LeftParenthesis
            | RightParenthesis | LeftBrace | RightBrace | LessThan | MoreThan | Assign | Bang => 1,
            Equals | NotEquals | Append | Shell | If | Fi => 2,
            Then | Else => 4,
            Literal(s) | Path(s) | Flag(s) | Ident(s) | DoubleStr(s) | SingleStr(s)
            | FormatStr(s) => s.len(),
        }
    }
}

pub trait IntoLexer<'l> {
    fn lexer(&'l self) -> Lexer<'l>;
}
impl<'l> IntoLexer<'l> for &str {
    fn lexer(&'l self) -> Lexer<'l> { Lexer::new(self) }
}

#[derive(Debug)]
pub struct Lexer<'l> {
    line: &'l [u8],
    idx: Cell<usize>
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a &'a str) -> Self {
        Lexer { line: s.as_bytes(), idx: Cell::new(0) }
    }

    pub fn fstring_iter(&'a mut self) -> FstringLexer<'a> {
        FstringLexer(self)
    }
}

trait Scanner<'l> {
    fn peeker(&self) -> impl FnMut() -> Option<char>;
    
    fn current(&self) -> Option<char>;
    fn advance(&self, n: usize) -> &Self;
    fn advance_while(&self, predicate: impl Fn(&char) -> bool) -> usize;
    fn read_str_while(&self, predicate: impl Fn(&char) -> bool) -> &'l str;

    const IS_PATH: fn(&char) -> bool = |&c| Self::IS_LITERAL(&c) || c == '*';
    const IS_LITERAL: fn(&char) -> bool = |&c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '/');
    const IS_IDENT: fn(&char) -> bool = |&c| c.is_ascii_alphanumeric() || matches!(c, '_' | '@' | '$' | '!');
    const IS_FSTRING: fn(&char) -> bool = |&c| !matches!(c, '`' | '$');

    fn read_path(&self) -> &'l str { self.read_str_while(Self::IS_PATH) }
    fn read_literal(&self) -> &'l str { self.read_str_while(Self::IS_LITERAL) }
    fn read_ident(&self) -> &'l str { self.advance(1).read_str_while(Self::IS_IDENT) }
    fn read_fstring(&self) -> &'l str { self.read_str_while(Self::IS_FSTRING) }
    fn read_delim_str(&self, delim: char) -> &'l str { 
        let ret = self.advance(1).read_str_while(|&c| c != delim);
        self.advance(1);
        ret
    }
}

impl<'l> Scanner<'l> for Lexer<'l> {
    fn peeker(&self) -> impl FnMut() -> Option<char> {
        let mut pos = self.idx.get();
        move || {
            let ret = self.line.get(pos).map(|c| *c as char);
            pos += 1;
            ret
        }
    }

    fn current(&self) -> Option<char> {
        self.line.get(self.idx.get()).map(|c| *c as char)
    }

    fn advance(&self, n: usize) -> &Self {
        self.idx.set(self.idx.get() + n);
        self
    }

    fn advance_while(&self, predicate: impl Fn(&char) -> bool) -> usize {
        let n = self.line[self.idx.get()..].iter().take_while(|&&c| predicate(&(c as char))).count();
        self.advance(n);
        n
    }

    fn read_str_while(&self, predicate: impl Fn(&char) -> bool) -> &'l str {
        let start = self.idx.get();
        self.advance(1).advance_while(predicate);
        unsafe { str::from_utf8_unchecked(&self.line[start..self.idx.get()]) }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use Token::*;

        let _space_before = self.advance_while(|&c| c == ' ');
        if let Some('#') = self.current() {
            self.advance_while(|&c| c != '\n');
        }
        // println!("{} spaces before token", space_before);

        let mut peek = self.peeker();
        peek().map(|c: char| -> Token {
            let (token, len) = match c {
                '=' if Some('=') == peek() => (Equals, 2),
                '!' if Some('=') == peek() => (NotEquals, 2),
                '>' if Some('>') == peek() => (Append, 2),
                '$' if Some('(') == peek() => (Shell, 2),
                ';' => (Semicolon, 1),
                '|' => (Pipe, 1),
                '`' => (BackTick, 1),
                '*' if peek().is_some_and(char::is_whitespace) => (Asterisk, 1),
                '$' if peek().is_some_and(char::is_whitespace) => (Dollar, 1),
                '(' => (LeftParenthesis, 1),
                ')' => (RightParenthesis, 1),
                '{' => (LeftBrace, 1),
                '}' => (RightBrace, 1),
                '<' => (LessThan, 1),
                '>' => (MoreThan, 1),
                '=' => (Assign, 1),
                '!' => (Bang, 1),
                '.' | '~' | '/' | '*' => return Path(self.read_path()),
                'a'..='z' | 'A'..='Z' | '_' => {
                    return match self.read_literal() {
                        "if" => If,
                        "then" => Then,
                        "else" => Else,
                        "fi" => Fi,
                        lit => Literal(lit),
                    }
                }
                '$' => return Ident(self.read_ident()),
                '-' => return Flag(self.read_literal()),
                '"' => return DoubleStr(self.read_delim_str('"')),
                '\'' => return SingleStr(self.read_delim_str('\'')),
                c => {
                    panic!("Invalid character! [{c}]");
                    // (Invalid, 1)
                }
            };
            // Advance the iterator to the current character
            self.advance(len);
            token
        })
    }
}

pub struct FstringLexer<'f>(&'f mut Lexer<'f>);
impl<'a> Iterator for FstringLexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use Token::*;

        let mut peek = self.0.peeker();
        peek().map(|c| {
            let (tok, len) = match (c, peek().unwrap_or_default()) {
                ('`', _) => (BackTick, 1),
                ('$', '(') => (Shell, 2),
                ('$', _) => return Ident(self.0.read_ident()),
                _ => return FormatStr(self.0.read_fstring()),
            };
            self.0.advance(len);
            tok
        })
    }
}

// TEMP
pub trait Tokenize<'t> {
    fn tokenize(&'t self) -> Vec<Token<'t>>;
}

// TEMP
impl<'t> Tokenize<'t> for &str {
    fn tokenize(&'t self) -> Vec<Token<'t>> {
        self.lexer().collect()
    }
}
