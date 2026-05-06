use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum StateError {
    
    #[error("Internal error!")]
    InternalError
}