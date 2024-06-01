use std::borrow::BorrowMut;
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
 * <pipeline> ::= <cmd> { `|` <cmd> }
 * <cmd>  ::= ( <literal> | <substitute> ) { <arg> }
 * <arg> ::= <literal> | <quote> | <substitute>
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
use ast::ParseTree;

mod ast {
    use super::*;
    use std::{
        fmt::{Debug, Display},
        ops::BitOr,
    };

    impl Debug for ParseTree {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            fn helper(tree: &ParseTree, depth: usize, r_l: u16) -> String {
                let mut branches = "".to_string();

                for i in 0..depth {
                    if (1 & (r_l >> i)) == 1 {
                        branches.push_str("│   ")
                    } else {
                        branches.push_str("    ")
                    }
                }
                match tree {
                    ParseTree::Pipe(l, r) => format!(
                        "PIPE\n{branches}├──{}\n{branches}└──{}",
                        helper(l, depth + 1, r_l | (1 << depth)),
                        helper(r, depth + 1, r_l)
                    ),
                    ParseTree::Command { name, args } => format!(
                        "Command\n{branches}├──{}\n{branches}└──Arg(s):{}",
                        helper(name, depth + 1, r_l | (1 << depth)),
                        args.split_last()
                            .map(|(last, rest)| {
                                rest.into_iter()
                                    .map(|arg| {
                                        format!(
                                            "\n{branches}    ├── {}",
                                            helper(arg, depth + 2, r_l | (1 << depth + 1))
                                        )
                                    })
                                    .collect::<Vec<String>>()
                                    .join("")
                                    + format!(
                                        "\n{branches}    └── {}",
                                        helper(last, depth + 1, r_l)
                                    )
                                    .as_str()
                            })
                            .unwrap_or_default()
                    ),
                    ParseTree::Literal(lit) => format!("Literal: {lit}"),
                    ParseTree::Identifier(id) => format!("{id}"),
                    ParseTree::String(s) => format!("\"{s}\""),
                }
            }

            write!(f, "{}", helper(self, 0, 0))
        }
    }

    impl Display for ParseTree {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ParseTree::Pipe(l, r) => write!(f, "[{l} | {r}]"),
                ParseTree::Command { name, args } => write!(
                    f,
                    "[ {} {{{}}} ]",
                    name.as_ref(),
                    args.iter()
                        .map(|arg| arg.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                ParseTree::Literal(lit) => write!(f, "{lit}"),
                ParseTree::Identifier(id) => todo!(),
                ParseTree::String(s) => write!(f, "\"{s}\""),
            }
        }
    }

    #[derive(Clone, PartialEq)]
    pub enum ParseTree {
        Pipe(Box<ParseTree>, Box<ParseTree>),
        Command {
            name: Box<ParseTree>,
            args: Vec<ParseTree>,
        },

        Literal(String),
        Identifier(String),
        String(String),
    }

    trait TreeBuilder {
        fn parse_pipe(&mut self) -> Box<ParseTree>;
        fn parse_command(&mut self) -> Box<ParseTree>;
        fn parse_args(&mut self) -> Vec<ParseTree>;
    }

    impl TreeBuilder for Peekable<IterMut<'_, Token>> {
        fn parse_pipe(&mut self) -> Box<ParseTree> {
            let mut tree = self.parse_command();

            while let Some(_) = self.next_if(|t| t.inner() == "|") {
                tree = Box::new(ParseTree::Pipe(tree, self.parse_command()));
            }
            tree
        }

        fn parse_command(&mut self) -> Box<ParseTree> {
            let Some(token) = self.next() else {
                panic!("INVALID TOKEN")
            };

            Box::new(ParseTree::Command {
                name: Box::new(match token {
                    Token::Literal(lit) => ParseTree::Literal(mem::take(lit)),
                    Token::Symbol(sym) if sym.as_str() == "$" => todo!(),
                    _ => todo!(),
                }),
                args: self.parse_args(),
            })
        }

        fn parse_args(&mut self) -> Vec<ParseTree> {
            let mut args = vec![];

            while let Some(token) = self.peek() {
                match token {
                    Token::Literal(_) => args.push(ParseTree::Literal(mem::take(
                        self.next().unwrap().inner_mut(),
                    ))),
                    _ => return args, // Token::Symbol(_) => todo!(),
                                      // Token::Identifier(_) => todo!(),
                                      // Token::Str(_) => todo!(),
                                      // Token::Comment(_) => unimplemented!(),
                                      // Token::Wildcard(_) => unimplemented!(),
                                      // Token::ControlOperator(_) => unimplemented!(),
                }
            }
            args
        }
    }

    impl From<Vec<Token>> for ParseTree {
        fn from(mut tokens: Vec<Token>) -> Self {
            let mut token_it = tokens.iter_mut().peekable();

            let line = token_it.parse_pipe();

            *line
        }
    }
}

pub trait Parse {
    fn parse(self) -> ParseTree;
}

impl Parse for Vec<Token> {
    fn parse(self) -> ParseTree {
        ParseTree::from(self)
    }
}
