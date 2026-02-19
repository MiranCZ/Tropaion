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
    Else,
    While,
    For,
    Break,
    Continue,
    Fn,
    Return,
    Struct,

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
    BitXor,
    Arrow, // ->

    PlusPlus,
    MinusMinus,
    RightRight,
    LeftLeft,

    BoolAnd,
    BoolOr,

    Assign, //=

    // shorthand assigns
    PlusAssign,
    DashAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    RightRightAssign,
    LeftLeftAssign,
    VertBarAssign,
    AmpersandAssign,
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
            Else => "else",
            While => "while",
            For => "for",
            Break => "break",
            Continue => "continue",
            Fn => "fn",
            Return => "return",
            Struct => "struct",
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
            BoolAnd => "&&",
            BoolOr => "||",
            BitXor => "^",
            Assign => "=",
            PlusAssign => "+=",
            DashAssign => "-=",
            StarAssign => "*=",
            SlashAssign => "/=",
            PercentAssign => "%=",
            RightRightAssign => ">>=",
            LeftLeftAssign => "<<=",
            VertBarAssign => "|=",
            AmpersandAssign => "&=",
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
