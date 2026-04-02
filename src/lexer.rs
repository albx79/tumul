use std::collections::VecDeque;
use std::str::Lines;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Ident(String),
    Number(f64),
    String(String),
    Tag(String),
    LParen,
    RParen,
    Comma,
    Plus,
    Minus,
    Times,
    Div,
    Arrow,
    Lambda,
    Assign,
    Match,
    Bang,
    Colon,
    DotDot,
    Indent,
    Dedent,
    Newline,
    Underscore,
    Eof,
}

type TokenPos = (usize, Token, usize);

pub struct Lexer<'a> {
    current_pos: usize,
    lines: Lines<'a>,
    indentation_stack: Vec<usize>,
    pending_tokens: VecDeque<TokenPos>,
    eof_emitted: bool,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = TokenPos;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.eof_emitted {
            Some(self.next_token())
        } else {
            None
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            current_pos: 0,
            lines: input.lines(),
            indentation_stack: vec![0], // stack bottom is 0
            pending_tokens: VecDeque::new(),
            eof_emitted: false,
        }
    }

    pub fn next_token(&mut self) -> TokenPos {
        // 1. Return any pending INDENT/DEDENT/content/NEWLINE tokens
        if let Some(token) = self.pending_tokens.pop_front() {
            return token;
        }
        // 2. If at EOF, emit all Dedents and then Eof
        if self.eof_emitted {
            return (self.current_pos, Token::Eof, self.current_pos);
        }

        // 3. Otherwise, read the next non-blank, non-comment line
        while let Some(line) = self.lines.next() {
            let original = line;
            let line = line.trim_end(); // remove trailing whitespace
            self.current_pos += original.len() - line.len();
            // skip empty lines entirely (they don't affect indentation)
            if line.trim().is_empty() {
                self.current_pos += line.len();
                continue;
            }

            // Leading spaces/tabs
            let indent = original
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .fold(0, |acc, c| acc + if c == '\t' { 4 } else { 1 }); // treat tab as 4 (or whatever)

            // 4. Emit INDENT/DEDENT before anything else
            let current_indent = *self.indentation_stack.last().unwrap();
            if indent > current_indent {
                self.indentation_stack.push(indent);
                self.pending_tokens.push_back((self.current_pos, Token::Indent, self.current_pos + indent - current_indent));
            } else if indent < current_indent {
                while indent < *self.indentation_stack.last().unwrap() {
                    self.indentation_stack.pop();
                    self.pending_tokens.push_back((self.current_pos, Token::Dedent, self.current_pos + current_indent - indent));
                }
            }

            // 5. Now emit tokens for this line, then NEWLINE
            let tokens = lex_line_tokens(line, &mut self.current_pos);
            for tok in tokens {
                self.pending_tokens.push_back(tok);
            }
            self.pending_tokens.push_back((self.current_pos, Token::Newline, self.current_pos.inc()));

            // 6. Return the next token (will be INDENT/DEDENTs first if present)
            return self.pending_tokens.pop_front().unwrap();
        }

        // 7. At EOF: emit remaining DEDENTs before the Eof token
        while self.indentation_stack.len() > 1 {
            self.indentation_stack.pop();
            self.pending_tokens.push_back((self.current_pos, Token::Dedent, self.current_pos));
        }
        self.pending_tokens.push_back((self.current_pos, Token::Eof, self.current_pos));
        self.eof_emitted = true;
        self.pending_tokens.pop_front().unwrap()
    }
}

fn lex_line_tokens(line: &str, pos: &mut usize) -> Vec<TokenPos> {
    // Simple hand-rolled scanner for Tumul tokens
    let mut tokens: Vec<TokenPos> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        // Skip whitespace inside the line (indentation was handled before)
        if c.is_whitespace() {
            i += 1;
            pos.inc();
            continue;
        }
        // Numbers
        if c.is_ascii_digit() {
            let mut j = i;
            while j < chars.len() && chars[j].is_ascii_digit() {
                j += 1;
            }
            if j < chars.len() && chars[j] == '.' {
                j += 1;
                while j < chars.len() && chars[j].is_ascii_digit() {
                    j += 1;
                }
            }
            let num_str = &line[i..j];
            let num = num_str.parse::<f64>().unwrap();
            tokens.push(pos.tok_i_j(Token::Number(num), i, j));
            i = j;
            continue;
        }
        // Identifier or keyword
        if c.is_ascii_alphabetic() || c == '_' {
            let mut j = i;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            let id = &line[i..j];
            let len = j - i;
            tokens.push((*pos, Token::Ident(id.to_string()), *pos + len));
            *pos += len;
            i = j;
            continue;
        }
        // Tag (e.g., 'ok)
        if c == '\'' {
            let mut j = i + 1;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            let tag = &line[i + 1..j];
            let len = j - i;
            tokens.push((*pos, Token::Tag(tag.to_string()), *pos + len));
            *pos += len;
            i = j;
            continue;
        }
        // String literal
        if c == '"' {
            let mut j = i + 1;
            while j < chars.len() {
                if chars[j] == '"' && chars[j - 1] != '\\' {
                    break;
                }
                j += 1;
            }
            let string_lit = &line[i + 1..j];
            tokens.push(pos.tok_i_j(Token::String(string_lit.replace("\\\"", "\"")), i, j));
            i = j + 1;
            continue;
        }
        // Operators and punctuation
        match c {
            '(' => {
                tokens.push(pos.tok_inc(Token::LParen));
                i += 1;
            }
            ')' => {
                tokens.push(pos.tok_inc(Token::RParen));
                i += 1;
            }
            ',' => {
                tokens.push(pos.tok_inc(Token::Comma));
                i += 1;
            }
            '=' => {
                tokens.push(pos.tok_inc(Token::Assign));
                i += 1;
            }
            '?' => {
                tokens.push(pos.tok_inc(Token::Match));
                i += 1;
            }
            '!' => {
                tokens.push(pos.tok_inc(Token::Bang));
                i += 1;
            }
            ':' => {
                tokens.push(pos.tok_inc(Token::Colon));
                i += 1;
            }
            '\\' => {
                tokens.push(pos.tok_inc(Token::Lambda));
                i += 1;
            }
            '+' => {
                tokens.push(pos.tok_inc(Token::Plus));
                i += 1;
            }
            '-' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(pos.tok_inc_by(Token::Arrow, 2));
                    i += 2;
                } else {
                    tokens.push(pos.tok_inc(Token::Minus));
                    i += 1;
                }
            }
            '*' => {
                tokens.push(pos.tok_inc(Token::Times));
                i += 1;
            }
            '/' => {
                tokens.push(pos.tok_inc(Token::Div));
                i += 1;
            }
            '.' => {
                if i + 1 < chars.len() && chars[i + 1] == '.' {
                    tokens.push(pos.tok_inc_by(Token::DotDot, 2));
                    i += 2;
                } else {
                    *pos += 1;
                    i += 1;
                }
            }
            _ => {
                *pos += 1;
                i += 1;
            }
        }
    }
    tokens
}

#[test]
fn test() {
//     let src = r#"
// foo = (x, y)
// bar = \n -> (n, ())
// baz = foo ! bar(2)
// sequence = \t -> t ?
//   () -> 0
//   (h, t..) -> h + sequence(t..)
// other() !
//   fnc 1
//   fnc 2 ?
//     'ok -> -1.2
//     'bad -> 3.14
// "#;
    let src = "2 + 3 + 4 + 5";
    let mut lexer = Lexer::new(src);
    loop {
        let tok = lexer.next_token();
        print!("{:?} ", tok);
        if tok.1 == Token::Newline {
            println!()
        }
        if tok.1 == Token::Eof {
            break;
        }
    }
}

trait Inc {
    fn inc(&mut self) -> usize;
    fn tok_i_j(&mut self, tok: Token, i: usize, j: usize) -> TokenPos;
    fn tok_inc(&mut self, tok: Token) -> TokenPos;
    fn tok_inc_by(&mut self, tok: Token, n: usize) -> TokenPos;
}

impl Inc for usize {
    fn inc(&mut self) -> usize {
        *self += 1;
        *self
    }

    fn tok_i_j(&mut self, tok: Token, i: usize, j: usize) -> TokenPos {
        let old = *self;
        *self += j - i;
        (old, tok, *self)
    }

    fn tok_inc(&mut self, tok: Token) -> TokenPos {
        self.tok_inc_by(tok, 1)
    }

    fn tok_inc_by(&mut self, tok: Token, n: usize) -> TokenPos {
        let old = *self;
        *self += n;
        (old, tok, *self)
    }
}