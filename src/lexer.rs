use crate::lexer::token::Token::{Comment, Identifier, MultilineComment, NumberFloatLiteral, NumberIntLiteral, StringLiteral, EOF};
use crate::lexer::token::{SimpleToken, Token};
use std::cmp::Reverse;
use strum::IntoEnumIterator;

pub mod token;

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
                return MultilineComment(self.next_multiline_comment().unwrap());
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

        let mut candidates: Vec<SimpleToken> = SimpleToken::iter().collect();
        candidates.sort_by_key(|k| Reverse(k.string_representation().len()));

        self.pos -= 1;

        let mut text = String::new();

        let start_pos = self.pos;
        let mut biggest_match: Option<SimpleToken> = None;
        let mut match_at = self.pos;

        while let Ok(next) = self.next_char() {
            text.push(next);

            let mut continue_loop = false;
            for x in candidates.iter() {
                if x.string_representation() == text.as_str() {

                    if biggest_match.is_none() || x.string_representation().len() > biggest_match.unwrap().string_representation().len() {
                        biggest_match = Some(*x);
                        match_at = self.pos;
                    }

                    continue_loop = true;
                }

                if x.string_representation().starts_with(text.as_str()) {
                    continue_loop = true;
                }
            }
            if !continue_loop {
                break;
            }
        }

        self.pos = match_at;
        if let Some(result) = biggest_match {
            return Token::SimpleToken(result);
        }

        self.pos = start_pos;

        if let Some(identifier) = self.next_word() {
            return Identifier(identifier);
        }

        EOF
    }

}
