use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "tumul.pest"]
pub struct TumulParser;
