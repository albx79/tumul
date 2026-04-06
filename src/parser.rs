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
    env_logger::init();
    let srcs = &["
foo = 3
bar = 4
baz = (2 + 3) * 4 / (foo - bar)
baz
",
    r#"
my_string = "foobar"
a_tag = 'ok
a_record = (foo: 1, bar: "hello", baz: (1, 2, 3), foo, bar, baz)
()
"#,
    "
empty_tuple = ()
and_trailing_comma = (1, 2, 3,)

42
",
    r#"
a_block =
  foo = "value"
  bar = 42
  bar + 2

row_with_block = (foo: 1, bar: 2, baz:
  tmp1 = 3
  tmp2 = 4
  tmp1 * tmp2
)

foo + bar + 2.0 * baz / "another string, with \"quotes\""
"#];
    println!("Testing parser with {} source snippets", srcs.len());
    for (i, src) in srcs.iter().enumerate() {
        println!("{i}> `{src}`");
        let ast = parse(src);
        dbg!(ast);
    }
}
