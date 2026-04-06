#[derive(Debug)]
pub enum Ast {
    Program(Vec<Ast>, Box<Ast>),
    Block(Vec<Ast>, Box<Ast>),
    Assign(String, Box<Ast>),
    Plus(Box<Ast>, Box<Ast>),
    Minus(Box<Ast>, Box<Ast>),
    Times(Box<Ast>, Box<Ast>),
    Div(Box<Ast>, Box<Ast>),
    Row(Vec<RowField>),
    Num(f64),
    Str(String),
    Var(String),
    Tag(String)
}

#[derive(Debug)]
pub enum RowField {
    Named(String, Box<Ast>),
    Unnamed(Box<Ast>),
}
