use crate::lexer::Token;

#[derive(Debug)]
pub enum Ast {
    Program(Vec<Ast>, Box<Ast>),
    Assign(Token, Box<Ast>),
    Plus(Box<Self>, Box<Self>),
    Num(Token),
    Var(Token),
}