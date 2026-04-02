use lalrpop_util::lalrpop_mod;

lalrpop_mod!(tumul);

use tumul::ProgramParser as Parser;

use crate::{ast::Ast, lexer::Lexer};

pub fn parse(input: &str) -> Ast {
    let lexer = Lexer::new(input);
    let parsed = Parser::new().parse(lexer);
    parsed.unwrap()
}

#[test]
fn test_parser() {
    let src = r"
foo = 3
bar = 4

foo + bar + 2.0
";
    let ast = parse(src);
    dbg!(ast);
}