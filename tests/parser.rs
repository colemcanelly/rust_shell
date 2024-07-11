use lang::parser::{
    Tree::*,
    Parse
};

use lang::lexer::Token as Tok;

// fn tmp() {
    

//     ;

//     ;

//     Command {
//         name: Box::new(Literal("alias")),
//         args: vec![Literal("colors")],
//     };



//     Command {
//         name: Box::new(Literal("ls")),
//         args: vec![
//             Subshell(Box::new(Command {
//                 name: Box::new(Literal("echo")),
//                 args: vec![Literal("-a")],
//             })),
//             Literal("-l"),
//         ],
//     };

//     ;

//     Command {
//         name: Box::new(Literal("echo")),
//         args: vec![
//             Literal("-e"),
//             Subshell(Box::new(Command {
//                 name: Box::new(Literal("ls")),
//                 args: vec![Literal("-a")],
//             })),
//             String(r#""Directory""#),
//             Subshell(Box::new(Pipe(
//                 Box::new(Pipe(
//                     Box::new(Command {
//                         name: Box::new(Literal("history")),
//                         args: vec![],
//                     }),
//                     Box::new(Command {
//                         name: Box::new(Literal("grep")),
//                         args: vec![Literal("git")],
//                     }),
//                 )),
//                 Box::new(Command {
//                     name: Box::new(Literal("sort")),
//                     args: vec![Literal("-u"), Literal("-k2")],
//                 }),
//             ))),
//         ],
//     };
// }


#[test]
fn command_arguments() {
    // Programs without arguments
    assert_eq!(
        vec![Tok::Literal("~/bin/ansi_colors")].parse(),
        Command {
            name: Box::new(Literal("~/bin/ansi_colors")),
            args: vec![],
        }
    );
    // Programs with arguments
    assert_eq!(
        vec![
            Tok::Literal("ls"),
            Tok::Literal("-F"),
            Tok::Literal("--group-directories-first")
        ].parse(),
        Command {
            name: Box::new(Literal("ls")),
            args: vec![Literal("-F"), Literal("--group-directories-first")],
        }
    );
    assert_eq!(
        vec![
            Tok::Literal("xclip"),
            Tok::Literal("-selection"),
            Tok::Literal("c"),
            Tok::Literal("-o")
        ].parse(),
        Command {
            name: Box::new(Literal("xclip")),
            args: vec![Literal("-selection"), Literal("c"), Literal("-o")],
        }
    );
}

#[test]
fn quotes() {
    // String with no spaces inside
    assert_eq!(
        vec![
            Tok::Literal("grep"),
            Tok::Symbol("\""),
            Tok::Str(":Zone.Identifier"),
            Tok::Symbol("\"")
        ].parse(),
        Command {
            name: Box::new(Literal("grep")),
            args: vec![String(r#"":Zone.Identifier""#)],
        }
    );

    assert_eq!(
        vec![
            Tok::Literal("echo"),
            Tok::Symbol("\""),
            Tok::Str("My name is Cole McAnelly"),
            Tok::Symbol("\"")
        ].parse(),
        Command {
            name: Box::new(Literal("echo")),
            args: vec![String(r#""My name is Cole McAnelly""#)],
        }
    );

    // Single quotes with assignment operator
    // assert_eq!(
    //     vec![
    //         Tok::Literal("alias"),
    //         Tok::Literal("colors"),
    //         Tok::Symbol("="),
    //         Tok::Symbol("\'"),
    //         Tok::Str("~/bin/ansi_colors"),
    //         Tok::Symbol("\'")
    //     ].parse(),
    //     todo!()
    // );
    /*
    Command {
        name: Box::new(Literal("alias")),
        args: vec![Literal("colors")],
    }
    */

    // Double quoted String with internal spaces, and assignment operator
    // assert_eq!(
    //     vec![
    //         Tok::Literal("MY_VAR"),
    //         Tok::Symbol("="),
    //         Tok::Symbol("\""),
    //         Tok::Str("this is the value of my variable"),
    //         Tok::Symbol("\"")
    //     ].parse(),
    //     todo!()
    // );
}

#[test]
fn pipes() {
    // Pipes with spaces in between
    assert_eq!(
        vec![
            Tok::Literal("history"),
            Tok::Symbol("|"),
            Tok::Literal("grep"),
            Tok::Literal("git"),
            Tok::Symbol("|"),
            Tok::Literal("xargs"),
            Tok::Literal("rm")
        ].parse(),
        Pipe(
            Box::new(Pipe(
                Box::new(Command { 
                    name: Box::new(Literal("history")),
                    args: vec![]
                }),
                Box::new(Command {
                    name: Box::new(Literal("grep")),
                    args: vec![Literal("git")]
                })
            )),
            Box::new(Command {
                name: Box::new(Literal("xargs")),
                args: vec![Literal("rm")]
            })
        )
    );
    // assert_eq!(
    //     vec![
    //         Tok::Literal("ls"),
    //         Tok::Wildcard("./src/*.rs"),
    //         Tok::Symbol("|"),
    //         Tok::Literal("xargs"),
    //         Tok::Literal("basename"),
    //         Tok::Literal("-s"),
    //         Tok::Literal(".rs")
    //     ].parse(),
    //     todo!()
    // );
}

// #[test]
// fn io_redirections() {
//     assert_eq!(
//         vec![
//             Tok::Literal("cat"),
//             Tok::Symbol("<<"),
//             Tok::Literal("EOF"),
//             Tok::Symbol(">"),
//             Tok::Literal("file"),
//             Tok::Symbol("|"),
//             Tok::Literal("wc"),
//             Tok::Literal("-c"),
//             Tok::Symbol("|"),
//             Tok::Literal("tr"),
//             Tok::Literal("-d"),
//             Tok::Symbol("\""),
//             Tok::Str(" "),
//             Tok::Symbol("\""),
//             Tok::Symbol(">"),
//             Tok::Literal("file2")
//         ].parse(),
//         todo!()
//     );

//     assert_eq!(
//         vec![
//             Tok::Literal("echo"),
//             Tok::Symbol("\""),
//             Tok::Str("This is Cole McAnelly's file, and I am writing my name inside of it!!"),
//             Tok::Symbol("\""),
//             Tok::Symbol(">>"),
//             Tok::Literal("my_file")
//         ].parse(),
//         todo!()
//     )
// }

// #[test]
// fn variables() {
//     assert_eq!(
//         vec![Tok::Literal("echo"), Tok::Symbol("$"), Tok::Identifier("VAR")].parse(),
//         todo!()
//     );

//     assert_eq!(
//         vec![
//             Tok::Literal("echo"),
//             Tok::Symbol("\""),
//             Tok::Str("this is "),
//             Tok::Symbol("$"),
//             Tok::Identifier("VAR"),
//             Tok::Str(" right here"),
//             Tok::Symbol("\"")
//         ].parse(),
//         todo!()
//     );
// }

#[test]
fn parenthesis() {
    // Pipes with spaces in between
    // todo!("SUBPROCESS TOKENIZING", );
    assert_eq!(
        vec![
            Tok::Literal("echo"),
            Tok::Symbol("$"),
            Tok::Symbol("("),
            Tok::Literal("ls"),
            Tok::Literal("-a"),
            Tok::Symbol(")")
        ].parse(),
        Command {
            name: Box::new(Literal("echo")),
            args: vec![
                Subshell(Box::new(Command {
                    name: Box::new(Literal("ls")),
                    args: vec![Literal("-a")],
                }))
            ],
        }
    );
    // assert_eq!(
    //     vec![
    //         Tok::Literal("echo"),
    //         Tok::Literal("-e"),
    //         Tok::Symbol("\""),
    //         Tok::Str(r"Here are the contents of the directory: [\n"),
    //         Tok::Symbol("$"),
    //         Tok::Symbol("("),
    //         Tok::Literal("ls"),
    //         Tok::Literal("-a"),
    //         Tok::Symbol(")"),
    //         Tok::Str(r"\n]"),
    //         Tok::Symbol("\""),
    //     ].parse(),
    //     todo!()
    // );

    // assert_eq!(
    //     vec![
    //         Tok::Literal("echo"),
    //         Tok::Symbol("\""),
    //         Tok::Str(""),
    //         Tok::Symbol("$"),
    //         Tok::Symbol("("),
    //         Tok::Literal("ls"),
    //         Tok::Literal("-a"),
    //         Tok::Symbol(")"),
    //         Tok::Str(""),
    //         Tok::Symbol("\""),
    //     ].parse(),
    //     todo!()
    // );
}

#[test]
fn complex() {
    // assert_eq!(
    //     vec![
    //         Tok::Literal("ls"),
    //         Tok::Literal("-l"),
    //         Tok::Symbol("\'"),
    //         Tok::Str("file name"),
    //         Tok::Symbol("\'"),
    //         Tok::Symbol("|"),
    //         Tok::Literal("grep"),
    //         Tok::Literal("test"),
    //         Tok::Symbol("$"),
    //         Tok::Identifier("VAR"),
    //         Tok::Comment("# This is a comment")
    //     ].parse(),
    //     todo!()
    // );

    assert_eq!(
        vec![
            Tok::Literal("find"),
            Tok::Literal("."),
            Tok::Literal("-type"),
            Tok::Literal("f"),
            Tok::Symbol("|"),
            Tok::Literal("grep"),
            Tok::Symbol("\""),
            Tok::Str(":Zone.Identifier"),
            Tok::Symbol("\""),
            Tok::Symbol("|"),
            Tok::Literal("xargs"),
            Tok::Literal("rm")
        ].parse(),
        Pipe(
            Box::new(Pipe(
                Box::new(Command {
                    name: Box::new(Literal("find")),
                    args: vec![Literal("."), Literal("-type"), Literal("f")],
                }),
                Box::new(Command {
                    name: Box::new(Literal("grep")),
                    args: vec![String(r#"":Zone.Identifier""#)],
                }),
            )),
            Box::new(Command {
                name: Box::new(Literal("xargs")),
                args: vec![Literal("rm")],
            }),
        )
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
    // , )}
}
