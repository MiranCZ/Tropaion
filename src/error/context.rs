use std::fmt::{Debug, Display, Formatter, LowerExp};
use crate::error::ok;

#[derive(Debug)]
pub struct ErrorContext<T> {
    pub error: T,
    pub span: Span,
    pub message: Option<String>
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Span {
    pub line: u32
}

impl Span {
    pub fn line(line: u32) -> Self {
        Self {
            line
        }
    }

    pub fn combined(a: Span, b: Span) -> Self {
        // TODO properly implement later
        Span::line(a.line)
    }
}

impl <T: Display> Display for ErrorContext<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)?;

        f.write_str(" on line ")?;

        Display::fmt(&self.span, f)?;

        ok()
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.line, f)
    }
}

impl <T> ErrorContext<T> {

    pub fn new(error: T, line: u32) -> Self {
        Self {
            error,
            span: Span::line(line),
            message: None
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);

        self
    }
}