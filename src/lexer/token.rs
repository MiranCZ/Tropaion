use crate::lexer::SimpleToken::*;
use strum_macros::EnumIter;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Unknown,
    
    SimpleTokenType(SimpleToken),

    Identifier(String),
    NumberIntLiteral(i64),
    NumberFloatLiteral(f32),
    StringLiteral(String),

    Comment(String),
    MultilineComment(String),

    EOF,
}



#[derive(Debug, PartialEq, Copy, Clone, Hash, Ord, PartialOrd, Eq, EnumIter)]
pub enum SimpleToken {
    Null,

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
    Pub,
    Priv,
    Fn,
    Init,
    Return,
    Struct,
    Enum,

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

    Question,
    TwoExcl,
    TwoQuestion,
    QuestionDot,

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
            Null => "null",
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
            Pub => "pub",
            Priv => "priv",
            Init => "init",
            Fn => "fn",
            Return => "return",
            Struct => "struct",
            Enum => "enum",
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
            Question => "?",
            TwoExcl => "!!",
            TwoQuestion => "??",
            QuestionDot => "?.",
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
