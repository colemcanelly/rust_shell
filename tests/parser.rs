use lang::parser::{Tree, Tree::*};

fn tmp() {
    Command {
        name: Box::new(Literal("~/bin/ansi_colors".into())),
        args: vec![],
    };

    Command {
        name: Box::new(Literal("ls".into())),
        args: vec![
            Literal("-F".into()),
            Literal("--group-directories-first".into()),
        ],
    };

    Command {
        name: Box::new(Literal("xclip".into())),
        args: vec![
            Literal("-selection".into()),
            Literal("c".into()),
            Literal("-o".into()),
        ],
    };

    Command {
        name: Box::new(Literal("alias".into())),
        args: vec![Literal("colors".into())],
    };

    Pipe(
        Box::new(Pipe(
            Box::new(Command {
                name: Box::new(Literal("history".into())),
                args: vec![],
            }),
            Box::new(Command {
                name: Box::new(Literal("grep".into())),
                args: vec![Literal("git".into())],
            }),
        )),
        Box::new(Command {
            name: Box::new(Literal("xargs".into())),
            args: vec![Literal("rm".into())],
        }),
    );

    Command {
        name: Box::new(Literal("echo".into())),
        args: vec![Subshell(Box::new(Command {
            name: Box::new(Literal("ls".into())),
            args: vec![Literal("-a".into())],
        }))],
    };
}
