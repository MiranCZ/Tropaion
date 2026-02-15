#[derive(Debug, PartialEq)]
pub enum ParserError {
    EOFError,
    NUDMissing,
    UnexpectedToken
}