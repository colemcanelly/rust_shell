// use rush::lexer::*;
use lang::lexer_lifetimes::{
    IntoLexer,
    Token::{self, *},
};

#[inline]
fn lexer_tester(input: &str, expected: Vec<Token>) {
    assert_eq!(
        input.lexer().collect::<Vec<Token>>(),
        expected,
        "\"{input}\""
    )
}

#[inline]
fn fstring_tester(input: &str, expected: Vec<Token>) {
    assert_eq!(
        input.lexer().fstring_iter().collect::<Vec<Token>>(),
        expected,
        "\"{input}\""
    )
}

#[test]
fn command_arguments() {
    // Programs without arguments
    lexer_tester("~/bin/ansi_colors", vec![Path("~/bin/ansi_colors")]);
    // Programs with arguments
    lexer_tester(
        "ls -F --group-directories-first",
        vec![Literal("ls"), Flag("-F"), Flag("--group-directories-first")],
    );
    lexer_tester(
        "xclip -selection c -o",
        vec![
            Literal("xclip"),
            Flag("-selection"),
            Literal("c"),
            Flag("-o"),
        ],
    );
}

#[test]
fn quotes() {
    // String with no spaces inside
    lexer_tester(
        r#"grep ":Zone.Literal""#,
        vec![Literal("grep"), DoubleStr(":Zone.Literal")],
    );

    lexer_tester(
        r#"echo "My name is Cole McAnelly""#,
        vec![Literal("echo"), DoubleStr("My name is Cole McAnelly")],
    );

    // Single quotes with assignment operator
    lexer_tester(
        "alias colors='~/bin/ansi_colors'",
        vec![
            Literal("alias"),
            Literal("colors"),
            Assign,
            SingleStr("~/bin/ansi_colors"),
        ],
    );

    // Double quoted String with internal spaces, and assignment operator
    lexer_tester(
        r#"MY_VAR="this is the value of my variable""#,
        vec![
            Literal("MY_VAR"),
            Assign,
            DoubleStr("this is the value of my variable"),
        ],
    );
}

#[test]
fn pipes() {
    // Pipes with spaces in between
    lexer_tester(
        r#"history | grep git | xargs rm"#,
        vec![
            Literal("history"),
            Pipe,
            Literal("grep"),
            Literal("git"),
            Pipe,
            Literal("xargs"),
            Literal("rm"),
        ],
    );
    lexer_tester(
        "ls ./src/*.rs | xargs basename -s .rs",
        vec![
            Literal("ls"),
            Path("./src/*.rs"),
            Pipe,
            Literal("xargs"),
            Literal("basename"),
            Flag("-s"),
            Path(".rs"),
        ],
    );

    // Pipes without spaces
    lexer_tester(
        r#"history|grep git|xargs rm"#,
        vec![
            Literal("history"),
            Pipe,
            Literal("grep"),
            Literal("git"),
            Pipe,
            Literal("xargs"),
            Literal("rm"),
        ],
    );
}

#[test]
fn io_redirections() {
    lexer_tester(
        r#"program > log"#,
        vec![Literal("program"), MoreThan, Literal("log")],
    );

    lexer_tester(
        "shell <src",
        vec![Literal("shell"), LessThan, Literal("src")],
    );

    lexer_tester(
        "shell <src >log",
        vec![
            Literal("shell"),
            LessThan,
            Literal("src"),
            MoreThan,
            Literal("log"),
        ],
    );

    lexer_tester(
        r#"echo "This is Cole McAnelly's file, and I am writing my name inside of it!!" >> my_file"#,
        vec![
            Literal("echo"),
            DoubleStr("This is Cole McAnelly's file, and I am writing my name inside of it!!"),
            Append,
            Literal("my_file"),
        ],
    );

    lexer_tester(
        "awk '{print $1$11}' < test.txt",
        vec![
            Literal("awk"),
            SingleStr("{print $1$11}"),
            LessThan,
            Literal("test.txt"),
        ],
    )
}

#[test]
fn pipe_redirections() {
    lexer_tester(
        r#"history | wc -c | tr -d " " > file2"#,
        vec![
            Literal("history"),
            Pipe,
            Literal("wc"),
            Flag("-c"),
            Pipe,
            Literal("tr"),
            Flag("-d"),
            DoubleStr(" "),
            MoreThan,
            Literal("file2"),
        ],
    );

    lexer_tester(
        "awk '{print $1$11}' < test.txt | head -10 | tr a-z A-Z | sort > output.txt",
        vec![
            Literal("awk"),
            SingleStr("{print $1$11}"),
            LessThan,
            Literal("test.txt"),
            Pipe,
            Literal("head"),
            Flag("-10"),
            Pipe,
            Literal("tr"),
            Literal("a-z"),
            Literal("A-Z"),
            Pipe,
            Literal("sort"),
            MoreThan,
            Literal("output.txt"),
        ],
    )
}

#[test]
fn variables() {
    lexer_tester("echo $VAR", vec![Literal("echo"), Ident("VAR")]);

    lexer_tester("MY_VAR=$VAR", vec![Literal("MY_VAR"), Assign, Ident("VAR")]);
}

#[test]
fn parenthesis() {
    // Pipes with spaces in between
    lexer_tester(
        "echo $(ls -a)",
        vec![
            Literal("echo"),
            Shell,
            Literal("ls"),
            Flag("-a"),
            RightParenthesis,
        ],
    );

    lexer_tester(
        "RUST_FILES=$(ls ./src/*.rs | xargs basename -s .rs)",
        vec![
            Literal("RUST_FILES"),
            Assign,
            Shell,
            Literal("ls"),
            Path("./src/*.rs"),
            Pipe,
            Literal("xargs"),
            Literal("basename"),
            Flag("-s"),
            Path(".rs"),
            RightParenthesis
        ],
    );
}

#[test]
fn fstrings_with_variables() {
    fstring_tester(
        "`this is $VAR right here`",
        vec![
            BackTick,
            FormatStr("this is "),
            Ident("VAR"),
            FormatStr(" right here"),
            BackTick,
        ],
    );

    fstring_tester(
        "`$VAR that is the value of $OTHER plus $THING`",
        vec![
            BackTick,
            Ident("VAR"),
            FormatStr(" that is the value of "),
            Ident("OTHER"),
            FormatStr(" plus "),
            Ident("THING"),
            BackTick,
        ],
    );
}

// fstring_tester(
//     "`$(ls -a)`",
//     vec![
//         BackTick,
//         Shell,
//         Literal("ls"),
//         Flag("-a"),
//         RightParenthesis,
//         BackTick,
//     ],
// );

// fstring_tester(
//     "`Here are the contents of the directory: [\n$(ls -a)\n]`",
//     vec![
//         BackTick,
//         FormatStr(r"Here are the contents of the directory: [\n"),
//         Shell,
//         Literal("ls"),
//         Flag("-a"),
//         RightParenthesis,
//         FormatStr(r"\n]"),
//         BackTick,
//     ],
// );
