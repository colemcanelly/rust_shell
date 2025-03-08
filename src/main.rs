use std::error::Error;

// use std::io::{stdin, stdout, Write};

// const PROMPT: &'static str = "rush> ";

use lang::{
    lexer_lifetimes::*,
    // lexer::Tokenize,
    parser::Parse,
};

use stats_alloc::{Region, Stats, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() -> Result<(), Box<dyn Error>> {
    // let mut line = String::new();

    // print!("{PROMPT}");
    // stdout().flush()?;

    // stdin().read_line(&mut line)?;

    // let lines =  [" $", "  *", "   (", "    )", "     {", "}", "|", ";", "==", "=", "!", "!=", "<", ">", ">>"];
    // for line in lines {
    //     println!("{:?}({line})", line.lexer().next().unwrap());
    // }

    // println!("\n");
    // "$(){}|;===!!=foo>bar>>/bin<".lexer().for_each(|t| print!(" {:?} ", t));

    // println!("\n\n{:?}", "PATH=/usr/bin/ls".lexer().collect::<Vec<Token>>());
    // println!("\n{:?}", "~/bin/ansi_colors".lexer().collect::<Vec<Token>>());
    // println!("\n{:?}", "ls -F --group-directories-first".lexer().collect::<Vec<Token>>());
    // println!("\n{:?}", r#"grep ":Zone.Identifier""#.lexer().collect::<Vec<Token>>());

    // return Ok(());

    let results: &[Stats] = &[
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"~/bin/ansi_colors"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"ls -F --group-directories-first"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"cat << EOF > file | wc -c | tr -d " " > file2"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"xclip -selection c -o"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"grep ":Zone.Identifier""#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"MY_VAR="this is the value of my variable""#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"alias colors='~/bin/ansi_colors'"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"history | grep git | xargs rm"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"ls ./src/*.rs | xargs basename -s .rs"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"echo $(ls -a)"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"ls $(echo -a) -l"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* let tokens = */ r#"echo -e $(ls -a) "Directory" $(history | grep git | sort -u -k2)"#.tokenize()/* .parse()) */;
            // println!("\n{:?}", tokens);
            reg.change()
        },
    ];

    results
        .iter()
        .enumerate()
        .for_each(|(i, stats)| println!("Allocations for {}: {:#?}", i + 1, stats));

    // reg.change()

    Ok(())
}
