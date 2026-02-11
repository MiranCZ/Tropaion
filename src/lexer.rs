use std::cmp::Reverse;
use crate::lexer::symbols::{Keywords, MathOperators, Operators};
use crate::lexer::symbols::Operators::{AssignMath, Math};
use crate::lexer::token::Token;
use crate::lexer::token::Token::{Comment, Identifier, Keyword, NumberIntLiteral, NumberFloatLiteral, Operator, Separator, StringLiteral, EOF};

pub mod token;
pub mod symbols;

pub struct Lexer {
    str: Vec<char>,
    pos: usize
}

struct EOFError{
}

impl Lexer {

    pub fn new(str: String) -> Self {
        Self{
            str: str.chars().collect(),
            pos: 0
        }
    }


    fn next_char(&mut self) -> Result<char, EOFError> {
        if self.pos >= self.str.len() {
            return Err(EOFError{});
        }

        let ch = self.str[self.pos];
        self.pos += 1;

        Ok(ch)
    }

    fn peek_char(&self) -> Result<char, EOFError> {
        if self.pos >= self.str.len() {
            return Err(EOFError{})
        }

        Ok(self.str[self.pos])
    }

    fn next_line(&mut self) -> String {
        let mut res = String::new();
        while let Ok(next) = self.next_char() {
            if next == '\r' && self.next_char_if('\n') {
                break;
            }
            if next == '\n' {
                break;
            }

            res.push(next);
        }

        res
    }

    fn next_multiline_comment(&mut self) -> Option<String> {
        let mut res = String::new();
        let mut ended = false;

        while let Ok(next) = self.next_char() {
            if next == '*' && self.next_char_if('/') {
                ended = true;
                break;
            }
            res.push(next);
        }

        if ended {
            return Some(res);
        }

        None
    }

    fn next_word(&mut self) -> Option<String> {
        let mut res = String::new();
        while let Ok(next) = self.next_char() {
            if next == ' ' || (!next.is_alphanumeric() && next != '_')  {
                self.pos -= 1;
                break;
            }
            res.push(next);
        }

        if res.is_empty() {
            return None;
        }

        Some(res)
    }

    fn next_string(&mut self) -> Option<String> {
        if !self.next_char_if('"') {
            return None;
        }

        let mut res = String::new();
        let mut ended = false;

        while let Ok(next) = self.next_char() {
            if next == '"' {
                ended = true;
                break;
            }
            if next == '\\' && self.next_char_if('"') {
                res.push('\\');
                res.push('"');
                continue;
            }

            res.push(next);
        }

        if ended {
            return Some(res);
        }

        // TODO error
        None
    }

    fn next_char_if(&mut self, ch: char) -> bool {
        let next = self.peek_char();

        if let Ok(v) = next && v == ch {
            let consumed = self.next_char();

            assert!(consumed.is_ok());
            return true;
        }

        false
    }

    fn next_keyword(&mut self) -> Option<Keywords> {
        let start_pos = self.pos;

        let mut candidates = Vec::from(Keywords::values());
        candidates.sort_by_key(|k| Reverse(k.text().len()));

        let mut text = String::new();

        'charLoop:
        while let Ok(next) = self.next_char() {
            text.push(next);

            for x in candidates.iter() {
                if x.text() == text.as_str() {
                    return Some(*x);
                }

                if x.text().starts_with(text.as_str()) {
                    continue 'charLoop;
                }
            }

            break;
        }

        self.pos = start_pos;
        None
    }

    fn parse_math_operator(&mut self, ch: char) -> Option<MathOperators> {

        match ch {
            '+' => Some(MathOperators::Plus),
            '-' => Some(MathOperators::Minus),
            '*' => Some(MathOperators::Multiply),
            '/' => Some(MathOperators::Divide),
            '%' => Some(MathOperators::Modulo),
            '|' => Some(MathOperators::BitOr),
            '&' => Some(MathOperators::BitAnd),
            '^' => Some(MathOperators::BitXor),
            '>' if self.next_char_if('>') => {
                Some(MathOperators::ShiftRight)
            },
            '<' if self.next_char_if('<') => {
                Some(MathOperators::ShiftLeft)
            }
            _ => None
        }
    }

    fn next_number_literal(&mut self) -> Option<Token> {
        let mut had_dot = false;

        let mut res = String::new();
        while let Ok(next) = self.next_char() {
            if next == '.' {
                if had_dot {
                    self.pos -= 1;
                    break;
                }

                had_dot = true;
            }

            if !next.is_numeric() {
                self.pos -= 1;
                break;
            }
            res.push(next);
        }

        if res.is_empty() {
            return None;
        }

        if had_dot {
            return Some(NumberFloatLiteral(res.parse::<f32>().unwrap()));
        }

        Some(NumberIntLiteral(res.parse::<i32>().unwrap()))
    }

    pub fn read_next(&mut self) -> Token {

        let ch;
        loop {
            let opt = self.next_char();

            if let Err(_) = opt {
                return EOF;
            }
            let char = opt.ok().unwrap();

            if char.is_whitespace() || (char == '\r' && self.next_char_if('\n')) {
                continue;
            }

            ch = char;
            break;
        }

        if ch == '/' {
            if self.next_char_if('/') {
                return Comment(self.next_line());
            }

            if self.next_char_if('*') {
                return Comment(self.next_multiline_comment().unwrap());
            }
        }

        if ch == '"' {
            self.pos -= 1;
            return StringLiteral(self.next_string().unwrap());
        }

        if ch.is_numeric() {
            self.pos -= 1;
            return self.next_number_literal().unwrap();
        }

        let mut has_two_chars = |desired| -> bool {
            return ch == desired && self.next_char_if('+');
        };

        if has_two_chars('+') {
            return Operator(Operators::Increment);
        }
        if has_two_chars('-') {
            return Operator(Operators::Decrement);
        }
        if has_two_chars('|') {
            return Operator(Operators::Or);
        }
        if has_two_chars('&') {
            return Operator(Operators::And);
        }

        let math_op = self.parse_math_operator(ch);

        if let Some(operator) = math_op {
            if self.next_char_if('=') {
                return Operator(AssignMath(operator));
            }

            return Operator(Math(operator));
        }

        match ch {
            '!' => {
                return if self.next_char_if('=') {
                    Operator(Operators::NotEquals)
                } else {
                    Operator(Operators::Not)
                }
            },
            '=' => {
                return if self.next_char_if('=') {
                    Operator(Operators::Equals)
                } else {
                    Operator(Operators::Assign)
                }
            },
            '~' => return Operator(Operators::BitNot),
            '>' => return Operator(Operators::BiggerThan),
            '<' => return Operator(Operators::LowerThan),
            _ => {
            }
        };


        let separators = ['(', ')', '[', ']', '{', '}', ';', ',', ':'];

        if separators.contains(&ch) {
            return Separator(ch);
        }


        self.pos -= 1;

        if let Some(keyword) = self.next_keyword() {
            return Keyword(keyword);
        }

        if let Some(identifier) = self.next_word() {
            return Identifier(identifier);
        }

        EOF
    }

}
