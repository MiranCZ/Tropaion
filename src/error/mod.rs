use std::fmt::Debug;

pub mod lexer_error;
pub mod parser_error;
pub mod runtime_error;
pub mod compilation_error;
pub mod analysis_error;
pub mod context;
pub mod error_type;
pub mod state_error;

pub fn ok<T>() -> Result<(), T>{
    Ok(())
}
