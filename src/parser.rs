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
    let src = r#"
foo = 3
bar = 4
baz = (2 + 3) * 4 / (foo - bar)
my_string = "foobar"
a_tag = 'ok
a_record = (foo: 1, bar: "hello", baz: (1, 2, 3), foo, bar, baz)
empty_tuple = ()
and_trailing_comma = (1, 2, 3,)

foo + bar + 2.0 * baz / "another string, with \"quotes\""
"#;
    let ast = parse(src);
    dbg!(ast);
}
