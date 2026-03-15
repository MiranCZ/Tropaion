use thiserror::Error;
use crate::compiler::bytecode::ByteCode;
use crate::interpreter::value::{Value, ValueType};


// FIXME this is not runtime error related but overall helper
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ValueTypeVariant {
    Null, Int, Float, Address,
    
    Bool, Number, Nullable, Array, Function, Struct
}

impl ValueTypeVariant {
    pub fn of(typ: ValueType) -> Self {
        match typ {
            ValueType::Null => Self::Null,
            ValueType::Int => Self::Int,
            ValueType::Float => Self::Float,
            ValueType::Address => Self::Address
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("Stack underflow! {0}")]
    StackUnderflow(&'static str),

    #[error("Stack frame is missing!")]
    StackFrameMissing,
    
    #[error("Unexpected StackFrame instruction")]
    UnexpectedStackFrame,
   
    #[error("Expected stack frame after call, got {0:?} instead")]
    StackFrameExpected(ByteCode),

    #[error("Expected type {expected:?} got {got:?} instead")]
    TypeMismatch {
        expected: ValueTypeVariant,
        got: Value
    },
    
    #[error("Illegal assignment, expected {expected:?} got {got:?} instead, previous value was {previous:?}")]
    IllegalAssignment{
        expected: ValueType,
        got: Value,
        previous: Value
    },

    #[error("Attempted to return on empty callstack!")]
    EmptyCallstack,

    #[error("Goto instruction pointer underflow")]
    InstructionPtrUnderflow,

    #[error("Goto instruction pointer overflow")]
    InstructionPtrOverflow,
   
    #[error("Attempted dereferencing a null pointer")]
    NullPtrDeref,
   
    #[error("Offset for reference is bigger than its size ({offset} > {len})")]
    OffsetOutOfBounds{len: u32, offset: u32},
   
    #[error("Attempting to call a non-existent function '{0}'")]
    FunctionNotFound(String),

    #[error("Illegal allocation size of {0}")]
    IllegalAllocSize(i32)
}