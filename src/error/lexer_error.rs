use std::io::Error;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug, PartialEq)]
pub enum LexerError {
    EOFError,
    UnclosedComment,
    UnclosedString,
    IdentifierExpected,
    UnknownToken(char),
    NumberExpected(String),
    IntParseFail(ParseIntError),
    FloatParseFail(ParseFloatError),
    
}