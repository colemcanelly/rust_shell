use std::error::Error;

mod lexer;
// mod parser;
use lexer::Tokenize;
// use std::io::{stdin, stdout, Write};

// const PROMPT: &'static str = "rush> ";



fn main() -> Result<(), Box<dyn Error>> {
    // let mut line = String::new();
    
    // print!("{PROMPT}");
    // stdout().flush()?;
    
    // stdin().read_line(&mut line)?;
    
    
    // println!("{line}");

    println!("{:?}", r#"~/bin/ansi_colors"#.tokenize());
    println!("{:?}", r#"ls -F --group-directories-first"#.tokenize());
    println!("{:?}", r#"cat << EOF > file | wc -c | tr -d " " > file2"#.tokenize());
    println!("{:?}", r#"xclip -selection c -o"#.tokenize());
    println!("{:?}", r#"grep ":Zone.Identifier""#.tokenize());
    println!("{:?}", r#"MY_VAR="this is the value of my variable""#.tokenize());
    println!("{:?}", r#"alias colors='~/bin/ansi_colors'"#.tokenize());
    println!("{:?}", r#"history|grep git | xargs rm"#.tokenize());
    println!("{:?}", r#"ls ./src/*.rs | xargs basename -s .rs"#.tokenize());
    println!("{:?}", r#"echo $(ls -a)"#.tokenize());
    println!("{:?}", r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize());


    Ok(())
}



