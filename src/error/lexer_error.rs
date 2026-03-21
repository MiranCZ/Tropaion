use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;
use crate::error::Error;
use crate::lexer::token::Token;

#[derive(Error, Debug, PartialEq)]
pub enum LexerError {
    #[error("Unexpectedly reached end of file")]
    EOFError,
    #[error("Unclosed multi-line comment")]
    UnclosedComment,
    #[error("Unclosed string literal")]
    UnclosedString,
    #[error("Expected an identifier token, got {0:?} instead")]
    IdentifierExpected(Token),
    #[error("Found an unknown character token '{0}'")]
    UnknownToken(char),
    #[error("Expected a number, got {0} instead")]
    NumberExpected(String),
    #[error("Failed to parse int literal '{0}' ({1})")]
    IntParseFail(String, ParseIntError),
    #[error("Failed to parse float literal '{0}' ({1})")]
    FloatParseFail(String, ParseFloatError),
    
}

impl Error for LexerError {
}