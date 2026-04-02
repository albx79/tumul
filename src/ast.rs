use crate::lexer::Token;

#[derive(Debug)]
pub enum Ast {
    Plus(Box<Self>, Box<Self>),
    Num(Token),
}