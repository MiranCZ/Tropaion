use crate::lexer::SimpleToken::*;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    SimpleTokenType(SimpleToken),

    Identifier(String),
    NumberIntLiteral(i32),
    NumberFloatLiteral(f32),
    StringLiteral(String),

    Comment(String),
    MultilineComment(String),

    EOF,
}



#[derive(Debug, PartialEq, Copy, Clone, Hash, Ord, PartialOrd, Eq, EnumIter)]
pub enum SimpleToken {
    True,
    False,

    // keywords
    Const,
    Let,
    If,
    While,
    For,
    Break,
    Continue,
    Fn,
    Return,

    // symbols
    Semicolon,
    Colon,
    Dot,
    Comma,
    OpenBracket,
    CloseBracket,
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,

    Plus,
    Dash,
    Star,
    Slash,
    Percent,
    Exclamation,
    Tilde, // ~
    Ampersand,
    VerticalBar,
    Arrow, // ->

    PlusPlus,
    MinusMinus,
    RightRight,
    LeftLeft,

    BitAnd,
    BitOr,
    BitXor,

    Assign, //=

    // shorthand assigns
    PlusAssign,
    DashAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    RightRightAssign,
    LeftLeftAssign,
    BitOrAssign,
    BitAndAssign,
    BitXorAssign,

    // comparisons
    Equals,
    NotEquals,
    Less,
    LessEquals,
    Greater,
    GreaterEquals,

}


impl SimpleToken {

    pub fn string_representation(&self) -> &'static str {
        match self {
            True => "true",
            False => "false",
            Const => "const",
            Let => "let",
            If => "if",
            While => "while",
            For => "for",
            Break => "break",
            Continue => "continue",
            Fn => "fn",
            Return => "return",
            Semicolon => ";",
            Colon => ":",
            Dot => ".",
            Comma => ",",
            OpenBracket => "(",
            CloseBracket => ")",
            OpenCurly => "{",
            CloseCurly => "}",
            OpenSquare => "[",
            CloseSquare => "]",
            Plus => "+",
            Dash => "-",
            Star => "*",
            Slash => "/",
            Percent => "%",
            Exclamation => "!",
            Tilde => "~",
            Ampersand => "&",
            VerticalBar => "|",
            Arrow => "->",
            PlusPlus => "++",
            MinusMinus => "--",
            RightRight => ">>",
            LeftLeft => "<<",
            BitAnd => "&&",
            BitOr => "||",
            BitXor => "^",
            Assign => "=",
            PlusAssign => "+=",
            DashAssign => "-=",
            StarAssign => "*=",
            SlashAssign => "/=",
            PercentAssign => "%=",
            RightRightAssign => ">>=",
            LeftLeftAssign => "<<=",
            BitOrAssign => "|=",
            BitAndAssign => "&=",
            BitXorAssign => "^=",
            Equals => "==",
            NotEquals => "!=",
            Less => "<",
            LessEquals => "<=",
            Greater => ">",
            GreaterEquals => ">=",
        }
    }
}
