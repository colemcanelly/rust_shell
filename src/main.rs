use std::error::Error;

mod lexer;
mod parser;
use lexer::Tokenize;
use parser::Parse;
// use std::io::{stdin, stdout, Write};

// const PROMPT: &'static str = "rush> ";



fn main() -> Result<(), Box<dyn Error>> {
    // let mut line = String::new();
    
    // print!("{PROMPT}");
    // stdout().flush()?;
    
    // stdin().read_line(&mut line)?;
    
    
    // println!("{line}");

    println!("{:?}\n", r#"~/bin/ansi_colors"#.tokenize().parse());
    println!("{:?}\n", r#"ls -F --group-directories-first"#.tokenize().parse());
    // println!("{:?}\n", r#"cat << EOF > file | wc -c | tr -d " " > file2"#.tokenize().parse());
    println!("{:?}\n", r#"xclip -selection c -o"#.tokenize().parse());
    // println!("{:?}\n", r#"grep ":Zone.Identifier""#.tokenize().parse());
    // println!("{:?}\n", r#"MY_VAR="this is the value of my variable""#.tokenize().parse());
    println!("{:?}\n", r#"alias colors='~/bin/ansi_colors'"#.tokenize().parse());
    println!("{:?}\n", r#"history | grep git | xargs rm"#.tokenize().parse());
    // println!("{:?}\n", r#"ls ./src/*.rs | xargs basename -s .rs"#.tokenize().parse());
    println!("{:?}\n", r#"echo $(ls -a)"#.tokenize().parse());
    println!("{:?}\n", r#"ls $(echo -a) -l"#.tokenize().parse());
    println!("{:?}\n", r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize().parse());
    println!("{:?}\n", r#"echo -e $(ls -a) "Directory" $(history | grep git | sort -u -k2)"#.tokenize().parse());


    Ok(())
}



