use crate::lexer::token::Token::{Comment, Identifier, MultilineComment, NumberFloatLiteral, NumberIntLiteral, StringLiteral, EOF};
use crate::lexer::token::{SimpleToken, Token};
use std::cmp::Reverse;
use std::f32::consts::E;
use std::num::ParseFloatError;
use strum::IntoEnumIterator;
use crate::error::context::{ErrorContext, Errors, Span};
use crate::error::lexer_error::LexerError;

pub mod token;

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub span: Span
}

pub struct Lexer {
    str: Vec<char>,
    pos: usize,
    pub errors: Errors<LexerError>
}

impl Lexer {

    pub fn new(str: String) -> Self {
        Self{
            str: str.chars().collect(),
            pos: 0,
            errors: vec![]
        }
    }

    pub fn parse(&mut self) -> Vec<TokenInfo> {
        let mut res = vec![];
       
        loop {
            let before = self.pos;
            let read = self.read_next();

            let read = if let Ok(r) = read {
                r
            } else {
                self.errors.push(ErrorContext::new(read.err().unwrap(), before, self.pos));

                Token::Unknown
            };

            let next = TokenInfo{token: read, span: Span::new(before, self.pos)};

            if next.token == EOF {
                res.push(next);
                break;
            }
            
            res.push(next);
        }

        res
    }

    fn has_next(&self) -> bool {
        self.pos < self.str.len()
    }

    fn next_char(&mut self) -> Result<char, LexerError> {
        if self.pos >= self.str.len() {
            return Err(LexerError::EOFError);
        }

        let ch = self.str[self.pos];
        self.pos += 1;

        Ok(ch)
    }

    fn peek_char(&self) -> Result<char, LexerError> {
        if self.pos >= self.str.len() {
            return Err(LexerError::EOFError)
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

    fn next_multiline_comment(&mut self) -> Result<String, LexerError> {
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
            return Ok(res);
        }

        Err(LexerError::UnclosedComment)
    }

    fn try_next_word(&mut self) -> Option<String> {
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

    fn next_string(&mut self) -> Result<String, LexerError> {
        if !self.next_char_if('"') {
            panic!("Invalid `next_string` call");
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
            return Ok(res);
        }

        Err(LexerError::UnclosedString)
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


    fn next_number_literal(&mut self) -> Result<Token, LexerError> {
        let mut had_dot = false;

        let mut res = String::new();
        while let Ok(next) = self.next_char() {
            if next == '.' {
                if had_dot {
                    self.pos -= 1;
                    break;
                }

                had_dot = true;
                res.push(next);
                continue;
            }

            if !next.is_numeric() {
                self.pos -= 1;
                break;
            }
            res.push(next);
        }

        if res.is_empty() {
            return Err(LexerError::NumberExpected(res));
        }

        if had_dot {
            let parse_result = res.parse::<f32>();
            return match parse_result {
                Err(e) => Err(LexerError::FloatParseFail(res, e)),
                Ok(v) => Ok(NumberFloatLiteral(v))
            };
        }

        let parse_result = res.parse::<i64>();
        match parse_result {
            Err(e) => Err(LexerError::IntParseFail(res, e)),
            Ok(v) => Ok(NumberIntLiteral(v))
        }
    }

    pub fn read_next(&mut self) -> Result<Token, LexerError> {
        let ch;
        loop {
            if !self.has_next() {
                return Ok(EOF);
            }

            let char = self.next_char()?;

            if char.is_whitespace() || (char == '\r' && self.next_char_if('\n')) {
                continue;
            }

            ch = char;
            break;
        }

        if ch == '/' {
            if self.next_char_if('/') {
                return Ok(Comment(self.next_line()));
            }

            if self.next_char_if('*') {
                return Ok(MultilineComment(self.next_multiline_comment()?));
            }
        }

        if ch == '"' {
            self.pos -= 1;
            return Ok(StringLiteral(self.next_string()?));
        }

        if ch.is_numeric() {
            self.pos -= 1;
            return Ok(self.next_number_literal()?);
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
            return Ok(Token::SimpleTokenType(result));
        }

        self.pos = start_pos;

        if let Some(identifier) = self.try_next_word() {
            return Ok(Identifier(identifier));
        }

        self.pos += 1;
        Err(LexerError::UnknownToken(ch))
    }

    fn current_line(&self) -> u32 {
        self.str[0..self.pos].iter().filter(|ch| **ch == '\n').count() as u32 + 1
    }

}
