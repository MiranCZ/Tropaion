pub mod lexer_error;
pub mod parser_error;
pub mod runtime_error;

pub fn ok<T>() -> Result<(), T>{
    Ok(())
}