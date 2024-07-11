// use rush::lexer::*;
use lang::lexer::{
    Tokenize,
    Token::*
};

#[test]
fn command_arguments() {
    // Programs without arguments
    assert_eq!(
        "~/bin/ansi_colors".tokenize(),
        vec![Literal("~/bin/ansi_colors")]
    );
    // Programs with arguments
    assert_eq!(
        "ls -F --group-directories-first".tokenize(),
        vec![
            Literal("ls"),
            Literal("-F"),
            Literal("--group-directories-first")
        ]
    );
    assert_eq!(
        "xclip -selection c -o".tokenize(),
        vec![
            Literal("xclip"),
            Literal("-selection"),
            Literal("c"),
            Literal("-o")
        ]
    );
}

#[test]
fn tokenize_trait() {
    assert_eq!(
        "~/bin/ansi_colors".tokenize(),
        vec![Literal("~/bin/ansi_colors")]
    );
    assert_eq!(
        String::from("~/bin/ansi_colors").tokenize(),
        vec![Literal("~/bin/ansi_colors")]
    );
}

#[test]
fn quotes() {
    // String with no spaces inside
    assert_eq!(
        r#"grep ":Zone.Identifier""#.tokenize(),
        vec![
            Literal("grep"),
            Symbol("\""),
            Str(":Zone.Identifier"),
            Symbol("\"")
        ]
    );

    assert_eq!(
        r#"echo "My name is Cole McAnelly""#.tokenize(),
        vec![
            Literal("echo"),
            Symbol("\""),
            Str("My name is Cole McAnelly"),
            Symbol("\"")
        ]
    );

    // Single quotes with assignment operator
    assert_eq!(
        "alias colors='~/bin/ansi_colors'".tokenize(),
        vec![
            Literal("alias"),
            Literal("colors"),
            Symbol("="),
            Symbol("\'"),
            Str("~/bin/ansi_colors"),
            Symbol("\'")
        ]
    );

    // Double quoted String with internal spaces, and assignment operator
    assert_eq!(
        r#"MY_VAR="this is the value of my variable""#.tokenize(),
        vec![
            Literal("MY_VAR"),
            Symbol("="),
            Symbol("\""),
            Str("this is the value of my variable"),
            Symbol("\"")
        ]
    );
}

#[test]
fn pipes() {
    // Pipes with spaces in between
    assert_eq!(
        r#"history | grep git | xargs rm"#.tokenize(),
        vec![
            Literal("history"),
            Symbol("|"),
            Literal("grep"),
            Literal("git"),
            Symbol("|"),
            Literal("xargs"),
            Literal("rm")
        ]
    );
    assert_eq!(
        "ls ./src/*.rs | xargs basename -s .rs".tokenize(),
        vec![
            Literal("ls"),
            Wildcard("./src/*.rs"),
            Symbol("|"),
            Literal("xargs"),
            Literal("basename"),
            Literal("-s"),
            Literal(".rs")
        ]
    );

    // Pipes without spaces
    assert_eq!(
        r#"history|grep git|xargs rm"#.tokenize(),
        vec![
            Literal("history"),
            Symbol("|"),
            Literal("grep"),
            Literal("git"),
            Symbol("|"),
            Literal("xargs"),
            Literal("rm")
        ]
    );
}

#[test]
fn io_redirections() {
    assert_eq!(
        r#"cat << EOF > file | wc -c | tr -d " " > file2"#.tokenize(),
        vec![
            Literal("cat"),
            Symbol("<<"),
            Literal("EOF"),
            Symbol(">"),
            Literal("file"),
            Symbol("|"),
            Literal("wc"),
            Literal("-c"),
            Symbol("|"),
            Literal("tr"),
            Literal("-d"),
            Symbol("\""),
            Str(" "),
            Symbol("\""),
            Symbol(">"),
            Literal("file2")
        ]
    );

    assert_eq!(
        r#"echo "This is Cole McAnelly's file, and I am writing my name inside of it!!" >> my_file"#.tokenize(),
        vec![
            Literal("echo"),
            Symbol("\""),
            Str("This is Cole McAnelly's file, and I am writing my name inside of it!!"),
            Symbol("\""),
            Symbol(">>"),
            Literal("my_file")
        ]
    )
}

#[test]
fn variables() {
    assert_eq!(
        "echo $VAR".tokenize(),
        vec![Literal("echo"), Symbol("$"), Identifier("VAR")]
    );
    assert_eq!(
        r#"echo "this is $VAR right here""#.tokenize(),
        vec![
            Literal("echo"),
            Symbol("\""),
            Str("this is "),
            Symbol("$"),
            Identifier("VAR"),
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
        "echo $(ls -a)".tokenize(),
        vec![
            Literal("echo"),
            Symbol("$"),
            Symbol("("),
            Literal("ls"),
            Literal("-a"),
            Symbol(")")
        ]
    );
    assert_eq!(
        r#"echo -e "Here are the contents of the directory: [\n$(ls -a)\n]""#.tokenize(),
        vec![
            Literal("echo"),
            Literal("-e"),
            Symbol("\""),
            Str(r"Here are the contents of the directory: [\n"),
            Symbol("$"),
            Symbol("("),
            Literal("ls"),
            Literal("-a"),
            Symbol(")"),
            Str(r"\n]"),
            Symbol("\""),
        ]
    );
    assert_eq!(
        r#"echo "$(ls -a)""#.tokenize(),
        vec![
            Literal("echo"),
            Symbol("\""),
            Str(""),
            Symbol("$"),
            Symbol("("),
            Literal("ls"),
            Literal("-a"),
            Symbol(")"),
            Str(""),
            Symbol("\""),
        ]
    );
}

#[test]
fn complex() {
    assert_eq!(
        "ls -l 'file name' | grep test $VAR # This is a comment".tokenize(),
        vec![
            Literal("ls"),
            Literal("-l"),
            Symbol("\'"),
            Str("file name"),
            Symbol("\'"),
            Symbol("|"),
            Literal("grep"),
            Literal("test"),
            Symbol("$"),
            Identifier("VAR"),
            Comment("# This is a comment")
        ]
    );

    assert_eq!(
        r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize(),
        vec![
            Literal("find"),
            Literal("."),
            Literal("-type"),
            Literal("f"),
            Symbol("|"),
            Literal("grep"),
            Symbol("\""),
            Str(":Zone.Identifier"),
            Symbol("\""),
            Symbol("|"),
            Literal("xargs"),
            Literal("rm")
        ]
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
    // );
}
