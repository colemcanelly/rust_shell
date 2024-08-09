use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'t> {
    Invalid,

    // 1 wide
    // Newline,
    Semicolon,        // ;
    Pipe,             // |
    BackTick,         // `
    Asterisk,         // *
    Dollar,           // $
    LeftParenthesis,  // (
    RightParenthesis, // )
    LeftBrace,        // {
    RightBrace,       // }
    LessThan,         // <
    MoreThan,         // >
    Assign,           // =
    Bang,             // !

    // 2 wide
    Equals,    // ==
    NotEquals, // !=
    Append,    // >>
    Shell,     // $(

    // Reserved Keywords
    If,
    Then,
    Else,
    Fi,

    // Variadic
    Literal(&'t str),
    Path(&'t str),
    Flag(&'t str),      // -<flag> | --<flag>
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
    fn lexer(&'l self) -> Lexer<'l> {
        Lexer {
            line: self,
            it: self.chars(),
        }
    }
}

#[derive(Debug)]
pub struct Lexer<'l> {
    line: &'l &'l str,
    it: Chars<'l>,
}

impl<'a> Lexer<'a> {
    pub fn next_format_arg(&mut self) -> Option<Token<'a>> {
        use Token::*;

        let mut peek = self.it.clone();
        peek.next().map(|c| {
            let (tok, len) = match (c, peek.next().unwrap_or_default()) {
                ('`', _) => (BackTick, 1),
                ('$', '(') => (Shell, 2),
                ('$', _) => return Ident(self.it.read_ident().str()),
                _ => return FormatStr(self.it.read_fstring().str()),
            };
            self.it.by_ref().take(len).for_each(|_| {});
            tok
        })
    }
}

impl<'a> Iterator for Lexer<'a> { 
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        use Token::*;

        let space_before = self.it.clone().take_while(|&c| c == ' ').count();
        self.it.by_ref().take(space_before).for_each(|_| {});
        // println!("{} spaces before token", space_before);

        let mut peek = self.it.clone();
        peek.next().map(|c: char| -> Token {
            let (token, len) = match c {
                '=' if Some('=') == peek.next() => (Equals, 2),
                '!' if Some('=') == peek.next() => (NotEquals, 2),
                '>' if Some('>') == peek.next() => (Append, 2),
                '$' if Some('(') == peek.next() => (Shell, 2),
                ';' => (Semicolon, 1),
                '|' => (Pipe, 1),
                '`' => (BackTick, 1),
                '*' => (Asterisk, 1),
                '$' if peek.next().is_some_and(|w| w.is_whitespace()) => (Dollar, 1),
                '(' => (LeftParenthesis, 1),
                ')' => (RightParenthesis, 1),
                '{' => (LeftBrace, 1),
                '}' => (RightBrace, 1),
                '<' => (LessThan, 1),
                '>' => (MoreThan, 1),
                '=' => (Assign, 1),
                '!' => (Bang, 1),
                '.' | '~' | '/' => return Path(self.it.read_path().str()),
                'a'..='z' | 'A'..='Z' | '_' => {
                    return match self.it.read_literal().str() {
                        "if" => If,
                        "then" => Then,
                        "else" => Else,
                        "fi" => Fi,
                        lit => Literal(lit),
                    }
                }
                '$' => return Ident(self.it.read_ident().str()),
                '-' => return Flag(self.it.read_literal().str()),
                '"' => return DoubleStr(self.it.read().str_until('\"')),
                '\'' => return SingleStr(self.it.read().str_until('\'')),
                c => {
                    panic!("Invalid character! [{c}]");
                    // (Invalid, 1)
                }
            };
            // Advance the iterator to the current character
            self.it.by_ref().take(len).for_each(|_| {});
            token
        })
    }
}

trait Read: Clone + Iterator<Item = char> {
    const PATH: fn(&char) -> bool = |&c| c == '/' || Self::LITERAL(&c);
    const LITERAL: fn(&char) -> bool = |&c| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    const IDENT: fn(&char) -> bool = |&c| c.is_ascii_alphanumeric() || c == '_';
    const FSTRING: fn(&char) -> bool = |&c| c != '`' && c != '$';

    fn read(&mut self) -> (impl Iterator<Item = char>, &mut Self) {
        (self.clone().skip(1), self)
    }

    fn read_path(&mut self) -> (impl Iterator<Item = char>, &mut Self) {
        (self.clone().skip(1).take_while(Self::PATH), self)
    }

    fn read_literal(&mut self) -> (impl Iterator<Item = char>, &mut Self) {
        (self.clone().skip(1).take_while(Self::LITERAL), self)
    }

    fn read_ident(&mut self) -> (impl Iterator<Item = char>, &mut Self) {
        (self.clone().skip(1).take_while(Self::IDENT), self)
    }

    fn read_fstring(&mut self) -> (impl Iterator<Item = char>, &mut Self) {
        (self.clone().skip(1).take_while(Self::FSTRING), self)
    }
}
impl<'a> Read for Chars<'a> {}

trait ReadIters<'r> {
    fn str_until(self, esc: char) -> &'r str;
    fn str(self) -> &'r str;
}

impl<'r, I> ReadIters<'r> for (I, &mut Chars<'r>)
where
    I: Iterator<Item = char>,
{
    fn str_until(self, esc: char) -> &'r str {
        let mut stop = false;
        (
            self.0.map_while(move |c| -> Option<char> {
                (!stop).then_some({
                    stop = c == esc;
                    c
                })
            }),
            self.1,
        )
            .str()
    }

    fn str(self) -> &'r str {
        let (peek, it) = self;
        let s = it.as_str();
        let len = peek.fold(it.next().unwrap().len_utf8(), |l, c| {
            it.next();
            l + c.len_utf8()
        });
        &s[..len]
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
