use crate::lexer::Token;

#[derive(Debug)]
pub enum Ast {
    Program(Vec<Ast>, Box<Ast>),
    Assign(String, Box<Ast>),
    Plus(Box<Ast>, Box<Ast>),
    Minus(Box<Ast>, Box<Ast>),
    Times(Box<Ast>, Box<Ast>),
    Div(Box<Ast>, Box<Ast>),
    Num(f64),
    Str(String),
    Var(String),
    Tag(String)
}