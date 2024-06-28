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
use crate::lexer::Token;

use std::iter::Peekable;
use std::mem;

use std::slice::IterMut;

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
    fn parse_pipe(&mut self) -> Tree;
    fn parse_command(&mut self) -> Tree;
    fn parse_substitute(&mut self) -> Tree;
    fn parse_subshell(&mut self) -> Tree;
    fn parse_quote(&mut self, q: char) -> Tree;
}

impl TreeBuilder for Peekable<IterMut<'_, Token>> {
    fn parse_pipe(&mut self) -> Tree {
        let mut tree = self.parse_command();

        while let Some(_) = self.next_if(|t| t.inner() == "|") {
            tree = Tree::Pipe(Box::new(tree), Box::new(self.parse_command()));
        }
        tree
    }

    fn parse_command(&mut self) -> Tree {
        let token = self.next().expect("INVALID INPUT");

        Tree::Command {
            name: match token {
                Token::Literal(lit) => Box::new(Tree::Literal(mem::take(lit))),
                Token::Symbol(sym) if sym.as_str() == "$" => Box::new(self.parse_substitute()),
                _ => todo!("Implement error handling for invalid command tokens"),
            },
            args: {
                let mut args = vec![];

                while let Some(token) = self.peek() {
                    match token {
                        Token::Literal(_) => {
                            args.push(Tree::Literal(mem::take(self.next().unwrap().inner_mut())))
                        }
                        Token::Symbol(sym) if sym.as_str() == "$" => {
                            self.next();
                            args.push(self.parse_substitute());
                        }
                        Token::Symbol(sym) if sym.as_str() == "\"" => {
                            self.next();
                            args.push(self.parse_quote('\"'));
                        }
                        Token::Symbol(sym) if sym.as_str() == "\'" => {
                            self.next();
                            args.push(self.parse_quote('\''))
                        }
                        _ => break,
                        // Token::Wildcard(_) => unimplemented!(),
                        // Token::ControlOperator(_) => unimplemented!(),
                    }
                }
                args
            },
        }
    }

    fn parse_substitute(&mut self) -> Tree {
        let token = self.next().expect("INVALID INPUT");

        match token {
            Token::Identifier(id) => Tree::Identifier(mem::take(id)),
            Token::Symbol(sym) if sym.as_str() == "(" => {
                let subs = self.parse_subshell();

                self.peek()
                    .and_then(|t| (t.inner() == ")").then_some(()))
                    .expect("Unbalanced parenthesis!"); // eventually ok_or(...)?
                self.next();
                subs
            }
            _ => todo!("Implement error handling for invalid substitution token"), // Invalid $ character!
        }
    }

    fn parse_subshell(&mut self) -> Tree {
        Tree::Subshell(Box::new(self.parse_pipe()))
    }

    fn parse_quote(&mut self, q: char) -> Tree {
        let mut quoted = vec![];

        loop {
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
                    quoted.push(self.parse_substitute());
                }
                _ => todo!("Implement error handling for invalid quote contents"),
            };
            panic!("Parse error!")
        }
    }
}

impl From<Vec<Token>> for Tree {
    fn from(mut tokens: Vec<Token>) -> Self {
        tokens
            .iter_mut()
            .peekable()
            .parse_pipe()
    }
}

pub trait Parse {
    fn parse(self) -> Tree;
}

impl Parse for Vec<Token> {
    fn parse(self) -> Tree {
        Tree::from(self)
    }
}

impl std::fmt::Debug for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn vec_to_string(v: &Vec<Tree>, l_pad: String) -> String {
            let Some((last, rest)) = v.split_last() else {
                return "".into();
            };
            rest.into_iter().fold("".into(), |s, arg| {
                format!("{s}\n{l_pad}├──{}", helper(arg, format!("{l_pad}│   ")))
            }) + format!("\n{l_pad}└──{}", helper(last, format!("{l_pad}    "))).as_str()
        }

        fn helper(tree: &Tree, l_pad: String) -> String {
            match tree {
                Tree::Pipe(l, r) => format!(
                    "PIPE\n{l_pad}├──{}\n{l_pad}└──{}",
                    helper(l, format!("{l_pad}│   ")),
                    helper(r, format!("{l_pad}    "))
                ),
                Tree::Command { name, args } => format!(
                    "COMMAND\n{l_pad}├──{}\n{l_pad}└──ARGS{}",
                    helper(name, format!("{l_pad}│   ")),
                    vec_to_string(args, format!("{l_pad}    "))
                ),
                Tree::Subshell(line) => format!(
                    "SUBSHELL\n{l_pad}└──{}",
                    helper(line, format!("{l_pad}    "))
                ),
                Tree::Quote(_, v) => format!(
                    "QUOTE\n{l_pad}└──{}",
                    vec_to_string(v, format!("{l_pad}    "))
                ),
                Tree::Literal(lit) => format!("LITERAL: {lit}"),
                Tree::Identifier(id) => format!("IDENT: {id}"),
                Tree::String(s) => format!("STRING: {s}"),
            }
        }

        write!(f, "{}", helper(self, "".into()))
    }
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pipe(l, r) => write!(f, "Pipe(Box::new({l}), Box::new({r}))"),
            Self::Command { name, args } => write!(f,
                "Command {{ name: Box::new({}), args: vec![{}] }}",
                name.as_ref(),
                args.iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Quote(ch, v) => write!(f,
                "Quote({ch}, vec![{}])",
                v.iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("")
            ),
            Self::Subshell(s) => write!(f, "Subshell(Box::new({s}))"),
            Self::Literal(lit) => write!(f, "Literal(\"{lit}\".into())"),
            Self::Identifier(id) => write!(f, "Identifier(\"{id}\".into())"),
            Self::String(s) => write!(f, "String(r#\"{s}\"#.into())"),
        }
    }
    // fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //     match self {
    //         Self::Pipe(l, r) => write!(f, "{l} | {r}"),
    //         Self::Command { name, args } => write!(
    //             f,
    //             "{} {}",
    //             name.as_ref(),
    //             args.iter()
    //                 .map(|arg| arg.to_string())
    //                 .collect::<Vec<String>>()
    //                 .join(" ")
    //         ),
    //         Self::Subshell(_) => todo!(),
    //         Self::Literal(lit) => write!(f, "{lit}"),
    //         Self::Identifier(id) => write!(f, "${id}"),
    //         Self::String(s) => write!(f, "{s}"),
    //         Self::Quote(ch, v) => write!(
    //             f,
    //             "{ch}{}{ch}",
    //             v.iter()
    //                 .map(|s| s.to_string())
    //                 .collect::<Vec<String>>()
    //                 .join("")
    //         ),
    //     }
    // }
}
