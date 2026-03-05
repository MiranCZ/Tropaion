pub mod lexer_error;
pub mod parser_error;
pub mod runtime_error;
pub mod compilation_error;
pub mod analysis_error;
pub mod context;

pub fn ok<T>() -> Result<(), T>{
    Ok(())
}