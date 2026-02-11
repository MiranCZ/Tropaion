

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Operators {
    BitNot,
    Not, And, Or,
    Increment, Decrement,
    Math(MathOperators),
    Assign, AssignMath(MathOperators),
    Equals, NotEquals, BiggerThan, LowerThan
}

// FIXME these are actually operators with multiple operands, because `~=` should not be possible
#[derive(Debug)]
#[derive(PartialEq)]
pub enum MathOperators {
    // num math
    Plus, Minus, Multiply, Divide, Modulo, ShiftLeft, ShiftRight,
    // num bits
    BitAnd, BitOr, BitXor,
}


pub enum Separators {
    Semicolon
}

#[derive(Copy, Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Keywords {
    // data types
    Bool, Int, Float,
    // variables
    Const, Let,
    // loops
    If, While, For, Break, Continue,
    // functions
    Fn, Return
}

impl Keywords {

    pub fn values() -> [Keywords; 12] {
        use crate::lexer::symbols::Keywords::*;

        [Bool, Int, Float, Const, Let, If, While, For, Break, Continue, Fn, Return]
    }

    pub fn text(&self) -> &str {
        match self {
            Keywords::Bool => "bool",
            Keywords::Int => "int",
            Keywords::Float => "float",
            Keywords::Const => "const",
            Keywords::Let => "let",
            Keywords::If => "if",
            Keywords::While => "while",
            Keywords::For => "for",
            Keywords::Break => "break",
            Keywords::Continue => "continue",
            Keywords::Fn => "fn",
            Keywords::Return => "return"
        }
    }

}