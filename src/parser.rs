lalrpop_mod!(tumul); // generated file
use tumul::ProgramParser;

use crate::lexer::Lexer;

pub fn parse(input: &str) {
    let lexer = Lexer::new(input);
    let parsed = ProgramParser::new().parse(lexer);
    parsed
}