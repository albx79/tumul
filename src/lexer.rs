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
    Eof,
}

pub struct Lexer<'a> {
    lines: Lines<'a>,
    indentation_stack: Vec<usize>,
    pending_tokens: VecDeque<Token>,
    eof_emitted: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            lines: input.lines(),
            indentation_stack: vec![0], // stack bottom is 0
            pending_tokens: VecDeque::new(),
            eof_emitted: false,
        }
    }

    pub fn next_token(&mut self) -> Token {
        // 1. Return any pending INDENT/DEDENT/content/NEWLINE tokens
        if let Some(token) = self.pending_tokens.pop_front() {
            return token;
        }
        // 2. If at EOF, emit all Dedents and then Eof
        if self.eof_emitted {
            return Token::Eof;
        }

        // 3. Otherwise, read the next non-blank, non-comment line
        while let Some(line) = self.lines.next() {
            let original = line;
            let line = line.trim_end(); // remove trailing whitespace
            // skip empty lines entirely (they don't affect indentation)
            if line.trim().is_empty() {
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
                self.pending_tokens.push_back(Token::Indent);
            } else if indent < current_indent {
                while indent < *self.indentation_stack.last().unwrap() {
                    self.indentation_stack.pop();
                    self.pending_tokens.push_back(Token::Dedent);
                }
            }

            // 5. Now emit tokens for this line, then NEWLINE
            let tokens = lex_line_tokens(line);
            for tok in tokens {
                self.pending_tokens.push_back(tok);
            }
            self.pending_tokens.push_back(Token::Newline);

            // 6. Return the next token (will be INDENT/DEDENTs first if present)
            return self.pending_tokens.pop_front().unwrap();
        }

        // 7. At EOF: emit remaining DEDENTs before the Eof token
        while self.indentation_stack.len() > 1 {
            self.indentation_stack.pop();
            self.pending_tokens.push_back(Token::Dedent);
        }
        self.pending_tokens.push_back(Token::Eof);
        self.eof_emitted = true;
        self.pending_tokens.pop_front().unwrap()
    }
}

fn lex_line_tokens(line: &str) -> Vec<Token> {
    // Simple hand-rolled scanner for Tumul tokens
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        // Skip whitespace inside the line (indentation was handled before)
        if c.is_whitespace() {
            i += 1;
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
            tokens.push(Token::Number(num));
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
            tokens.push(Token::Ident(id.to_string()));
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
            tokens.push(Token::Tag(tag.to_string()));
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
            tokens.push(Token::String(string_lit.replace("\\\"", "\"")));
            i = j + 1;
            continue;
        }
        // Operators and punctuation
        match c {
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '=' => {
                tokens.push(Token::Assign);
                i += 1;
            }
            '?' => {
                tokens.push(Token::Match);
                i += 1;
            }
            '!' => {
                tokens.push(Token::Bang);
                i += 1;
            }
            ':' => {
                tokens.push(Token::Colon);
                i += 1;
            }
            '\\' => {
                tokens.push(Token::Lambda);
                i += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::Arrow);
                    i += 2;
                } else {
                    tokens.push(Token::Minus);
                    i += 1;
                }
            }
            '*' => {
                tokens.push(Token::Times);
                i += 1;
            }
            '/' => {
                tokens.push(Token::Div);
                i += 1;
            }
            '.' => {
                if i + 1 < chars.len() && chars[i + 1] == '.' {
                    tokens.push(Token::DotDot);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    tokens
}

#[test]
fn test() {
    let src = r#"
foo = (x, y)
bar = \n -> (n, ())
baz = foo ! bar(2)
sequence = \t -> t ?
  () -> 0
  (h, t..) -> h + sequence(t..)
other() !
  fnc 1
  fnc 2 ?
    'ok -> -1.2
    'bad -> 3.14
"#;
    let mut lexer = Lexer::new(src);
    loop {
        let tok = lexer.next_token();
        print!("{:?} ", tok);
        if tok == Token::Newline {
            println!()
        }
        if tok == Token::Eof {
            break;
        }
    }
}
