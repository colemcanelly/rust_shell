/**
 * Lexed Tokens:
 * Literal
 * Symbol
 * ControlOperator
 * Identifier
 * Wildcard
 * Str
 * Comment
 *
 * =========== EBNF ===========
 * My shell's grammar, BNF
 *
 * <pipeline> ::= <command> { `|` <command> }
 * <command>  ::= ( <literal> | <substitute> ) <args>
 * <args> ::= { <literal> | <quote> | <substitute> }
 *
 * <quote> ::= `"` <string> { <substitute> <string> } `"`
 * <substitute> ::= `$` (<ident> | <subshell>)
 * <subshell> ::= `(` <pipeline> `)`
 *
 *
 *
 * <string> == Str
 * <ident> == Identifier
 * <literal> == Literal
*/
use std::iter::Peekable;
use std::mem;

use std::slice::IterMut;

use crate::lexer::Token;

mod parse {
    use super::*;

    #[derive(Clone, PartialEq)]
    pub enum Tree {
        Pipe(Box<Tree>, Box<Tree>),
        Command { name: Box<Tree>, args: Vec<Tree> },

        Quote(char, Vec<Tree>),
        Subshell(Box<Tree>),

        Literal(String),
        Identifier(String),
        String(String),
    }

    trait TreeBuilder {
        fn parse_pipe(&mut self) -> Box<Tree>;
        fn parse_command(&mut self) -> Box<Tree>;
        fn parse_args(&mut self) -> Vec<Tree>;
        fn parse_substitute(&mut self) -> Box<Tree>;
        fn parse_subshell(&mut self) -> Box<Tree>;
        fn parse_quote(&mut self, q: char) -> Tree;
    }

    impl TreeBuilder for Peekable<IterMut<'_, Token>> {
        fn parse_pipe(&mut self) -> Box<Tree> {
            let mut tree = self.parse_command();

            while let Some(_) = self.next_if(|t| t.inner() == "|") {
                tree = Box::new(Tree::Pipe(tree, self.parse_command()));
            }
            tree
        }

        fn parse_command(&mut self) -> Box<Tree> {
            let token = self.next().expect("INVALID INPUT");

            Box::new(Tree::Command {
                name: match token {
                    Token::Literal(lit) => Box::new(Tree::Literal(mem::take(lit))),
                    Token::Symbol(sym) if sym.as_str() == "$" => self.parse_substitute(),
                    _ => todo!(),
                },
                args: self.parse_args(),
            })
        }

        fn parse_args(&mut self) -> Vec<Tree> {
            let mut args = vec![];

            while let Some(token) = self.peek() {
                match token {
                    Token::Literal(_) => {
                        args.push(Tree::Literal(mem::take(self.next().unwrap().inner_mut())))
                    }
                    Token::Symbol(sym) => match sym.as_str() {
                        "$" => {
                            self.next();
                            args.push(*self.parse_substitute());
                        }
                        "\"" => {
                            self.next();
                            args.push(self.parse_quote('\"'));
                        }
                        "\'" => {
                            self.next();
                            args.push(self.parse_quote('\''))
                        },
                        _ => return args,
                    },
                    _ => return args,
                    // Token::Wildcard(_) => unimplemented!(),
                    // Token::ControlOperator(_) => unimplemented!(),
                }
            }
            args
        }

        fn parse_substitute(&mut self) -> Box<Tree> {
            let token = self.next().expect("INVALID INPUT");

            match token {
                Token::Identifier(id) => Box::new(Tree::Identifier(mem::take(id))),
                Token::Symbol(sym) if sym.as_str() == "(" => {
                    let subs = self.parse_subshell();

                    self.peek()
                        .and_then(|t| (t.inner() == ")").then_some(()))
                        .expect("Unbalanced parenthesis!"); // eventually ok_or(...)?
                    self.next();
                    subs
                }
                _ => todo!(), // Invalid $ character!
            }
        }

        fn parse_subshell(&mut self) -> Box<Tree> {
            Box::new(Tree::Subshell(self.parse_pipe()))
        }

        fn parse_quote(&mut self, q: char) -> Tree {
            let mut quoted = vec![];

            let Some(Token::Str(string)) = self.next() else {
                panic!("Unbalanced quotation mark!")
            }; // mem::take(
            let Some(Token::Symbol(sym)) = self.next() else {
                panic!("Invalid string contents!")
            };

            match sym.as_str() {
                "\"" | "\'" => {
                    return if quoted.is_empty() {
                        string.insert(0, q);
                        string.push(q);
                        Tree::String(mem::take(string))
                    } else {
                        quoted.push(Tree::String(mem::take(string)));
                        Tree::Quote(q, mem::take(&mut quoted))
                    }
                }
                "$" => {
                    quoted.push(Tree::String(mem::take(string)));
                    quoted.push(*self.parse_substitute());
                }
                _ => todo!(),
            };
            panic!("Parse error!")
        }
    }

    impl From<Vec<Token>> for Tree {
        fn from(mut tokens: Vec<Token>) -> Self {
            let mut token_it = tokens.iter_mut().peekable();

            let line = token_it.parse_pipe();

            *line
        }
    }
}

pub trait Parse {
    fn parse(self) -> parse::Tree;
}

impl Parse for Vec<Token> {
    fn parse(self) -> parse::Tree {
        parse::Tree::from(self)
    }
}

impl std::fmt::Debug for parse::Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use parse::Tree;
        fn vec_to_string(v: &Vec<Tree>, deep: usize, bits: u16, l_pad: &String) -> String {
            v.split_last()
                .map(|(last, rest)| {
                    rest.into_iter()
                        .map(|arg| {
                            format!(
                                "\n{l_pad}    ├──{}",
                                helper(arg, deep + 2, bits | (1 << deep + 1))
                            )
                        })
                        .collect::<Vec<String>>()
                        .join("")
                        + format!("\n{l_pad}    └──{}", helper(last, deep + 2, bits)).as_str()
                })
                .unwrap_or_default()
        }
        fn helper(tree: &Tree, depth: usize, r_l_bitmask: u16) -> String {
            let mut l_pad = "".to_string();

            for i in 0..depth {
                if (1 & (r_l_bitmask >> i)) == 1 {
                    l_pad.push_str("│   ")
                } else {
                    l_pad.push_str("    ")
                }
            }
            match tree {
                Tree::Pipe(l, r) => format!(
                    "PIPE\n{l_pad}├──{}\n{l_pad}└──{}",
                    helper(l, depth + 1, r_l_bitmask | (1 << depth)),
                    helper(r, depth + 1, r_l_bitmask)
                ),
                Tree::Command { name, args } => format!(
                    "COMMAND\n{l_pad}├──{}\n{l_pad}└──args:{}",
                    helper(name, depth + 1, r_l_bitmask | (1 << depth)),
                    vec_to_string(args, depth, r_l_bitmask, &l_pad)
                ),
                Tree::Literal(lit) => format!("LITERAL: {lit}"),
                Tree::Identifier(id) => format!("IDENT: {id}"),
                Tree::String(s) => format!("STRING: {s}"),
                Tree::Subshell(line) => {
                    format!("SUBSHELL\n{l_pad}└──{}", helper(line, depth + 1, r_l_bitmask))
                }
                Tree::Quote(_, v) => format!(
                    "QUOTE\n{l_pad}└──{}",
                    vec_to_string(v, depth, r_l_bitmask, &l_pad)
                ),
            }
        }

        write!(f, "{}", helper(self, 0, 0))
    }
}

impl std::fmt::Display for parse::Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pipe(l, r) => write!(f, "{l} | {r}"),
            Self::Command { name, args } => write!(
                f,
                "{} {}",
                name.as_ref(),
                args.iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Self::Subshell(_) => todo!(),
            Self::Literal(lit) => write!(f, "{lit}"),
            Self::Identifier(id) => write!(f, "${id}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Quote(ch, v) => write!(f, "{ch}{}{ch}", v.iter().map(|s| s.to_string()).collect::<Vec<String>>().join("")),
        }
    }
}
