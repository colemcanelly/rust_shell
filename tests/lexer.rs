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
        vec![Literal("~/bin/ansi_colors".into())]
    );
    // Programs with arguments
    assert_eq!(
        "ls -F --group-directories-first".tokenize(),
        vec![
            Literal("ls".into()),
            Literal("-F".into()),
            Literal("--group-directories-first".into())
        ]
    );
    assert_eq!(
        "xclip -selection c -o".tokenize(),
        vec![
            Literal("xclip".into()),
            Literal("-selection".into()),
            Literal("c".into()),
            Literal("-o".into())
        ]
    );
}

#[test]
fn tokenize_trait() {
    assert_eq!(
        "~/bin/ansi_colors".tokenize(),
        vec![Literal("~/bin/ansi_colors".into())]
    );
    assert_eq!(
        String::from("~/bin/ansi_colors").tokenize(),
        vec![Literal("~/bin/ansi_colors".into())]
    );
}

#[test]
fn quotes() {
    // String with no spaces inside
    assert_eq!(
        r#"grep ":Zone.Identifier""#.tokenize(),
        vec![
            Literal("grep".into()),
            Symbol('\"'.into()),
            Str(":Zone.Identifier".into()),
            Symbol('\"'.into())
        ]
    );

    assert_eq!(
        r#"echo "My name is Cole McAnelly""#.tokenize(),
        vec![
            Literal("echo".into()),
            Symbol('\"'.into()),
            Str("My name is Cole McAnelly".into()),
            Symbol('\"'.into())
        ]
    );

    // Single quotes with assignment operator
    assert_eq!(
        "alias colors='~/bin/ansi_colors'".tokenize(),
        vec![
            Literal("alias".into()),
            Literal("colors".into()),
            Symbol("=".into()),
            Symbol('\''.into()),
            Str("~/bin/ansi_colors".into()),
            Symbol('\''.into())
        ]
    );

    // Double quoted String with internal spaces, and assignment operator
    assert_eq!(
        r#"MY_VAR="this is the value of my variable""#.tokenize(),
        vec![
            Literal("MY_VAR".into()),
            Symbol("=".into()),
            Symbol('\"'.into()),
            Str("this is the value of my variable".into()),
            Symbol('\"'.into())
        ]
    );
}

#[test]
fn pipes() {
    // Pipes with spaces in between
    assert_eq!(
        r#"history | grep git | xargs rm"#.tokenize(),
        vec![
            Literal("history".into()),
            Symbol("|".into()),
            Literal("grep".into()),
            Literal("git".into()),
            Symbol("|".into()),
            Literal("xargs".into()),
            Literal("rm".into())
        ]
    );
    assert_eq!(
        "ls ./src/*.rs | xargs basename -s .rs".tokenize(),
        vec![
            Literal("ls".into()),
            Wildcard("./src/*.rs".into()),
            Symbol("|".into()),
            Literal("xargs".into()),
            Literal("basename".into()),
            Literal("-s".into()),
            Literal(".rs".into())
        ]
    );

    // Pipes without spaces
    assert_eq!(
        r#"history|grep git|xargs rm"#.tokenize(),
        vec![
            Literal("history".into()),
            Symbol("|".into()),
            Literal("grep".into()),
            Literal("git".into()),
            Symbol("|".into()),
            Literal("xargs".into()),
            Literal("rm".into())
        ]
    );
}

#[test]
fn io_redirections() {
    assert_eq!(
        r#"cat << EOF > file | wc -c | tr -d " " > file2"#.tokenize(),
        vec![
            Literal("cat".into()),
            Symbol("<<".into()),
            Literal("EOF".into()),
            Symbol(">".into()),
            Literal("file".into()),
            Symbol("|".into()),
            Literal("wc".into()),
            Literal("-c".into()),
            Symbol("|".into()),
            Literal("tr".into()),
            Literal("-d".into()),
            Symbol('\"'.into()),
            Str(" ".into()),
            Symbol('\"'.into()),
            Symbol(">".into()),
            Literal("file2".into())
        ]
    );

    assert_eq!(
        r#"echo "This is Cole McAnelly's file, and I am writing my name inside of it!!" >> my_file"#.tokenize(),
        vec![
            Literal("echo".into()),
            Symbol('\"'.into()),
            Str("This is Cole McAnelly's file, and I am writing my name inside of it!!".into()),
            Symbol('\"'.into()),
            Symbol(">>".into()),
            Literal("my_file".into())
        ]
    )
}

#[test]
fn variables() {
    assert_eq!(
        "echo $VAR".tokenize(),
        vec![Literal("echo".into()), Symbol('$'.into()), Identifier("VAR".into())]
    );
    assert_eq!(
        r#"echo "this is $VAR right here""#.tokenize(),
        vec![
            Literal("echo".into()),
            Symbol('\"'.into()),
            Str("this is ".into()),
            Symbol('$'.into()),
            Identifier("VAR".into()),
            Str(" right here".into()),
            Symbol('\"'.into())
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
            Literal("echo".into()),
            Symbol('$'.into()),
            Symbol('('.into()),
            Literal("ls".into()),
            Literal("-a".into()),
            Symbol(')'.into())
        ]
    );
    assert_eq!(
        r#"echo -e "Here are the contents of the directory: [\n$(ls -a)\n]""#.tokenize(),
        vec![
            Literal("echo".into()),
            Literal("-e".into()),
            Symbol('\"'.into()),
            Str(r"Here are the contents of the directory: [\n".into()),
            Symbol('$'.into()),
            Symbol('('.into()),
            Literal("ls".into()),
            Literal("-a".into()),
            Symbol(')'.into()),
            Str(r"\n]".into()),
            Symbol('\"'.into()),
        ]
    );
    assert_eq!(
        r#"echo "$(ls -a)""#.tokenize(),
        vec![
            Literal("echo".into()),
            Symbol('\"'.into()),
            Str("".into()),
            Symbol('$'.into()),
            Symbol('('.into()),
            Literal("ls".into()),
            Literal("-a".into()),
            Symbol(')'.into()),
            Str("".into()),
            Symbol('\"'.into()),
        ]
    );
}

#[test]
fn complex() {
    assert_eq!(
        "ls -l 'file name' | grep test $VAR # This is a comment".tokenize(),
        vec![
            Literal("ls".into()),
            Literal("-l".into()),
            Symbol('\''.into()),
            Str("file name".into()),
            Symbol('\''.into()),
            Symbol("|".into()),
            Literal("grep".into()),
            Literal("test".into()),
            Symbol('$'.into()),
            Identifier("VAR".into()),
            Comment("# This is a comment".into())
        ]
    );

    assert_eq!(
        r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize(),
        vec![
            Literal("find".into()),
            Literal(".".into()),
            Literal("-type".into()),
            Literal("f".into()),
            Symbol("|".into()),
            Literal("grep".into()),
            Symbol('\"'.into()),
            Str(":Zone.Identifier".into()),
            Symbol('\"'.into()),
            Symbol("|".into()),
            Literal("xargs".into()),
            Literal("rm".into())
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
