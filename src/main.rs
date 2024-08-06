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

    let results: [Stats; 13] = [
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"~/bin/ansi_colors"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"ls -F --group-directories-first"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"cat << EOF > file | wc -c | tr -d " " > file2"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"xclip -selection c -o"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"grep ":Zone.Identifier""#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"MY_VAR="this is the value of my variable""#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"alias colors='~/bin/ansi_colors'"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"history | grep git | xargs rm"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"ls ./src/*.rs | xargs basename -s .rs"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"echo $(ls -a)"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"ls $(echo -a) -l"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"find . -type f | grep ":Zone.Identifier" | xargs rm"#.tokenize()/* .parse()) */;
            reg.change()
        },
        {
            let reg = Region::new(&GLOBAL);
            /* println!("\n{:?}", */
            r#"echo -e $(ls -a) "Directory" $(history | grep git | sort -u -k2)"#.tokenize()/* .parse()) */;
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
