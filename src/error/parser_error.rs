use thiserror::Error;
use crate::lexer::token::Token;

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("Unexpectedly reached end of file")]
    EOFError,
    #[error("NUD handler missing for {0:?} (either not implemented or invalid syntax)")]
    NUDMissing(Token),
    #[error("Expected token to be {expected:?}, got {actual:?} instead")]
    UnexpectedToken {
        expected: Token,
        actual: Token
    },
    #[error("Expected token to be {expected}, got {actual:?} instead")]
    MismatchedTokenType {
        expected: String,
        actual: Token
    },

    #[error("Clashing modifier")]
    ClashingModifier
}
