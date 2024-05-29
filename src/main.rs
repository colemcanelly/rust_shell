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

    println!("{:?}\n", r#"~/bin/ansi_colors"#.tokenize());
    println!("{:?}\n", r#"ls -F --group-directories-first"#.tokenize());
    println!("{:?}\n", r#"cat << EOF > file | wc -c | tr -d " " > file2"#.tokenize());
    println!("{:?}\n", r#"xclip -selection c -o"#.tokenize());
    println!("{:?}\n", r#"grep ":Zone.Identifier""#.tokenize());
    println!("{:?}\n", r#"MY_VAR="this is the value of my variable""#.tokenize());
    println!("{:?}\n", r#"alias colors='~/bin/ansi_colors'"#.tokenize());
    println!("{:?}\n", r#"history|grep git | xargs rm"#.tokenize());
    println!("{:?}\n", r#"ls ./src/*.rs | xargs basename -s .rs"#.tokenize());
    println!("{:?}\n", r#"echo $(ls -a)"#.tokenize());
    println!("{:?}\n", r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize());


    Ok(())
}



