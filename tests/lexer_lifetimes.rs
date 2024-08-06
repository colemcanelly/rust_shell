// use rush::lexer::*;
use lang::lexer_lifetimes::{
    IntoLexer, Lexer,
    Token::{self, *},
};

#[test]
fn command_arguments() {
    // Programs without arguments
    assert_eq!(
        "~/bin/ansi_colors".lexer().collect::<Vec<Token>>(),
        vec![Path("~/bin/ansi_colors")]
    );
    // Programs with arguments
    assert_eq!(
        "ls -F --group-directories-first"
            .lexer()
            .collect::<Vec<Token>>(),
        vec![Literal("ls"), Flag("-F"), Flag("--group-directories-first")]
    );
    assert_eq!(
        "xclip -selection c -o".lexer().collect::<Vec<Token>>(),
        vec![
            Literal("xclip"),
            Flag("-selection"),
            Literal("c"),
            Flag("-o")
        ]
    );
}

#[test]
fn quotes() {
    // String with no spaces inside
    assert_eq!(
        r#"grep ":Zone.Literal""#.lexer().collect::<Vec<Token>>(),
        vec![Literal("grep"), DoubleStr(":Zone.Literal"),]
    );

    assert_eq!(
        r#"echo "My name is Cole McAnelly""#.lexer().collect::<Vec<Token>>(),
        vec![Literal("echo"), DoubleStr("My name is Cole McAnelly"),]
    );

    // Single quotes with assignment operator
    assert_eq!(
        "alias colors='~/bin/ansi_colors'"
            .lexer()
            .collect::<Vec<Token>>(),
        vec![
            Literal("alias"),
            Literal("colors"),
            Assign,
            SingleStr("~/bin/ansi_colors"),
        ]
    );

    // Double quoted String with internal spaces, and assignment operator
    assert_eq!(
        r#"MY_VAR="this is the value of my variable""#
            .lexer()
            .collect::<Vec<Token>>(),
        vec![
            Literal("MY_VAR"),
            Assign,
            DoubleStr("this is the value of my variable"),
        ]
    );
}

#[test]
fn pipes() {
    // Pipes with spaces in between
    assert_eq!(
        r#"history | grep git | xargs rm"#.lexer().collect::<Vec<Token>>(),
        vec![
            Literal("history"),
            Pipe,
            Literal("grep"),
            Literal("git"),
            Pipe,
            Literal("xargs"),
            Literal("rm")
        ]
    );
    assert_eq!(
        "ls ./src/*.rs | xargs basename -s .rs"
            .lexer()
            .collect::<Vec<Token>>(),
        vec![
            Literal("ls"),
            Wildcard("./src/*.rs"),
            Pipe,
            Literal("xargs"),
            Literal("basename"),
            Flag("-s"),
            Path(".rs")
        ]
    );

    // Pipes without spaces
    assert_eq!(
        r#"history|grep git|xargs rm"#.lexer().collect::<Vec<Token>>(),
        vec![
            Literal("history"),
            Pipe,
            Literal("grep"),
            Literal("git"),
            Pipe,
            Literal("xargs"),
            Literal("rm")
        ]
    );
}

#[test]
fn io_redirections() {
    assert_eq!(
        r#"cat << EOF > file | wc -c | tr -d " " > file2"#
            .lexer()
            .collect::<Vec<Token>>(),
        vec![
            Literal("cat"),
            Symbol("<<"),
            Literal("EOF"),
            MoreThan,
            Literal("file"),
            Pipe,
            Literal("wc"),
            Flag("-c"),
            Pipe,
            Literal("tr"),
            Flag("-d"),
            DoubleStr(" "),
            MoreThan,
            Literal("file2")
        ]
    );

    assert_eq!(
        r#"echo "This is Cole McAnelly's file, and I am writing my name inside of it!!" >> my_file"#.lexer().collect::<Vec<Token>>(),
        vec![
            Literal("echo"),
            DoubleStr("This is Cole McAnelly's file, and I am writing my name inside of it!!"),
            Append,
            Literal("my_file")
        ]
    )
}

#[test]
fn variables() {
    assert_eq!(
        "echo $VAR".lexer().collect::<Vec<Token>>(),
        vec![Literal("echo"), Dollar, Literal("VAR")]
    );
    assert_eq!(
        r#"echo "this is $VAR right here""#.lexer().collect::<Vec<Token>>(),
        vec![
            Literal("echo"),
            Symbol("\""),
            Str("this is "),
            Dollar,
            Literal("VAR"),
            Str(" right here"),
            Symbol("\"")
        ]
    );
}

#[test]
fn parenthesis() {
    // Pipes with spaces in between
    // todo!("SUBPROCESS TOKENIZING");
    assert_eq!(
        "echo $(ls -a)".lexer().collect::<Vec<Token>>(),
        vec![
            Literal("echo"),
            Dollar,
            LeftParenthesis,
            Literal("ls"),
            Flag("-a"),
            RightParenthesis
        ]
    );
    assert_eq!(
        r#"echo -e "Here are the contents of the directory: [\n$(ls -a)\n]""#
            .lexer()
            .collect::<Vec<Token>>(),
        vec![
            Literal("echo"),
            Flag("-e"),
            Symbol("\""),
            Str(r"Here are the contents of the directory: [\n"),
            Dollar,
            LeftParenthesis,
            Literal("ls"),
            Flag("-a"),
            RightParenthesis,
            Str(r"\n]"),
            Symbol("\""),
        ]
    );
    assert_eq!(
        r#"echo "$(ls -a)""#.lexer().collect::<Vec<Token>>(),
        vec![
            Literal("echo"),
            Symbol("\""),
            Str(""),
            Dollar,
            LeftParenthesis,
            Literal("ls"),
            Flag("-a"),
            RightParenthesis,
            Str(""),
            Symbol("\""),
        ]
    );
}

#[test]
fn complex() {
    assert_eq!(
        "ls -l 'file name' | grep test $VAR # This is a comment"
            .lexer()
            .collect::<Vec<Token>>(),
        vec![
            Literal("ls"),
            Flag("-l"),
            SingleStr("file name"),
            Pipe,
            Literal("grep"),
            Literal("test"),
            Dollar,
            Literal("VAR"),
            // Comment("# This is a comment")
        ]
    );

    assert_eq!(
        r#"find . -type f | grep ":Zone.Literal" | xargs rm"#
            .lexer()
            .collect::<Vec<Token>>(),
        vec![
            Literal("find"),
            Path("."),
            Flag("-type"),
            Literal("f"),
            Pipe,
            Literal("grep"),
            DoubleStr(":Zone.Literal"),
            Pipe,
            Literal("xargs"),
            Literal("rm")
        ]
    );
    // assert_eq!(
    //     "stow $@ 2> >(grep -v 'BUG in find_stowed_path? Absolute/relative mismatch' 1>&2)".lexer().collect::<Vec<Token>>(),
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
