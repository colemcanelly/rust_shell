use crate::lexer::Token;



#[derive(Debug, Clone)]
pub enum Redirects {
    Single(String),
    Double(String)
}


#[derive(Debug, Clone)]
pub enum CommandParts {
    Keyword(String),
    Subshell(Pipeline)
}


#[derive(Default, Debug, Clone)]
pub struct Command {
    args: Vec<CommandParts>,
    file_in: Option<Redirects>,
    file_out: Option<Redirects>
}


#[derive(Default, Debug, Clone)]
pub struct Pipeline {
    line: Vec<Command>,
    in_redirect: Option<Redirects>,
    o_redirect: Option<Redirects>,
    e_redirect: Option<Redirects>,
}

pub trait Parse {
    fn parse(self) -> Pipeline;
}



impl Parse for Vec<Token> {
    fn parse(self) -> Pipeline {
        let command_line: Pipeline;
        for tok in self {
            match tok {
                '|' => 
            }
        }

        command_line
    }
}