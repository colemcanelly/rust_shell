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

use std::default::Default;
use std::iter::{FromIterator, Peekable};
use std::mem;

#[derive(Clone)]
pub enum Tree<T>
where
    T: AsRef<str> + Clone,
{
    Pipe(Box<Tree<T>>, Box<Tree<T>>),
    Command {
        name: Box<Tree<T>>,
        args: Vec<Tree<T>>,
    },

    Quote(char, Vec<Tree<T>>),
    Subshell(Box<Tree<T>>),

    Literal(T),
    Identifier(T),
    String(T),
}

trait TreeBuilder {
    fn parse_pipe(&mut self) -> Tree<String>;
    fn parse_command(&mut self) -> Tree<String>;
    fn parse_substitute(&mut self) -> Tree<String>;
    fn parse_subshell(&mut self) -> Tree<String>;
    fn parse_quote(&mut self, q: char) -> Tree<String>;
}

impl FromIterator<Token<String>> for Tree<String> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Token<String>>,
    {
        iter.into_iter().peekable().parse_pipe()
    }
}

impl<I> TreeBuilder for Peekable<I>
where
    I: Iterator<Item = Token<String>>,
{
    fn parse_pipe(&mut self) -> Tree<String> {
        let mut tree = self.parse_command();

        while let Some(_) = self.next_if(|t| t.inner() == "|") {
            tree = Tree::Pipe(Box::new(tree), Box::new(self.parse_command()));
        }
        tree
    }

    fn parse_command(&mut self) -> Tree<String> {
        Tree::Command {
            name: match self.next().expect("INVALID INPUT") {
                Token::Literal(mut lit) => Box::new(Tree::Literal(mem::take(&mut lit))),
                Token::Symbol(sym) if sym.as_str() == "$" => Box::new(self.parse_substitute()),
                _ => todo!("Implement error handling for invalid command tokens"),
            },
            args: {
                let mut args: Vec<Tree<String>> = vec![];

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

    fn parse_substitute(&mut self) -> Tree<String> {
        let token = self.next().expect("INVALID INPUT");

        match token {
            Token::Identifier(mut id) => Tree::Identifier(mem::take(&mut id)),
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

    fn parse_subshell(&mut self) -> Tree<String> {
        Tree::Subshell(Box::new(self.parse_pipe()))
    }

    fn parse_quote(&mut self, q: char) -> Tree<String> {
        let mut quoted = vec![];

        loop {
            let Some(Token::Str(mut string)) = self.next() else {
                panic!("Unbalanced quotation mark!")
            }; // mem::take(
            let Some(Token::Symbol(sym)) = self.next() else {
                panic!("Invalid string contents!")
            };

            match sym.as_ref() {
                "\"" | "\'" => {
                    return if quoted.is_empty() {
                        string.insert(0, q);
                        string.push(q);
                        Tree::String(mem::take(&mut string))
                    } else {
                        quoted.push(Tree::String(mem::take(&mut string)));
                        Tree::Quote(q, mem::take(&mut quoted))
                    }
                }
                "$" => {
                    quoted.push(Tree::String(mem::take(&mut string)));
                    quoted.push(self.parse_substitute());
                }
                _ => todo!("Implement error handling for invalid quote contents"),
            };
            panic!("Parse error!")
        }
    }
}

pub trait Parse {
    fn parse(self) -> Tree<String>;
}

impl<T> Parse for Vec<Token<T>>
where
    T: AsRef<str> + Clone + Default + ToString,
{
    fn parse(mut self) -> Tree<String> {
        self.iter_mut()
            .map(|t| t.inner_to_string())
            .collect()
    }
}

impl<T> std::fmt::Debug for Tree<T>
where
    T: AsRef<str> + Clone + Default + ToString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn vec_to_string<T>(v: &Vec<Tree<T>>, l_pad: String) -> String 
        where 
            T: AsRef<str> + Clone + Default + ToString
        {
            let Some((last, rest)) = v.split_last() else {
                return "".into();
            };
            rest.into_iter().fold("".into(), |s, arg| {
                format!("{s}\n{l_pad}├──{}", helper(arg, format!("{l_pad}│   ")))
            }) + format!("\n{l_pad}└──{}", helper(last, format!("{l_pad}    "))).as_str()
        }

        fn helper<T>(tree: &Tree<T>, l_pad: String) -> String 
        where 
            T: AsRef<str> + Clone + Default + ToString
        {
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
                Tree::Literal(lit) => format!("LITERAL: {}", lit.to_string()),
                Tree::Identifier(id) => format!("IDENT: {}", id.to_string()),
                Tree::String(s) => format!("STRING: {}", s.to_string()),
            }
        }

        write!(f, "{}", helper(self, "".into()))
    }
}

impl<T> std::fmt::Display for Tree<T>
where
    T: AsRef<str> + Clone + Default + ToString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pipe(l, r) => write!(f, "Pipe(Box::new({l}), Box::new({r}))"),
            Self::Command { name, args } => write!(
                f,
                "Command {{ name: Box::new({}), args: vec![{}] }}",
                name.as_ref(),
                args.iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Quote(ch, v) => write!(
                f,
                "Quote({ch}, vec![{}])",
                v.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join("")
            ),
            Self::Subshell(s) => write!(f, "Subshell(Box::new({s}))"),
            Self::Literal(lit) => write!(f, "Literal(\"{}\")", lit.to_string()),
            Self::Identifier(id) => write!(f, "Identifier(\"{}\")", id.to_string()),
            Self::String(s) => write!(f, "String(r#\"{}\"#)", s.to_string()),
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


impl<T, U> PartialEq<Tree<U>> for Tree<T>
where
    T: AsRef<str> + Clone,
    U: AsRef<str> + Clone,
{
    fn eq(&self, other: &Tree<U>) -> bool {
        use Tree::*;
        match (self, other) {
            (Pipe(a1, a2), Pipe(b1, b2)) => (a1.as_ref() == b1.as_ref()) && (a2.as_ref() == b2.as_ref()),
            (Command {name: a1, args: a2}, Command { name: b1, args: b2 }) => (a1.as_ref() == b1.as_ref()) && (a2 == b2),
            (Quote(a1, a2), Quote(b1, b2)) => (a1 == b1) && (a2 == b2),
            (Subshell(a), Subshell(b)) => a.as_ref() == b.as_ref(),
            (Literal(a), Literal(b))
            | (Identifier(a), Identifier(b))
            | (String(a), String(b)) => a.as_ref() == b.as_ref(),
            _ => false,
        }
    }
}