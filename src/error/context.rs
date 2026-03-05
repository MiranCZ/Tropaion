use std::cmp::{max, min};
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
    pub from: usize,
    pub to: usize
}

impl Span {
    pub fn new(from: usize, to: usize) -> Self {
        Self {
            from, to
        }
    }

    pub fn combined(a: Span, b: Span) -> Self {
        let from = min(a.from, b.from);
        let to = max(a.to, b. to);

        Span::new(from, to)
    }
}

// FIXME we need the input text to process errors
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
        Debug::fmt(&self.to, f)?;
        Debug::fmt("-", f)?;
        Debug::fmt(&self.from, f)
    }
}

impl <T> ErrorContext<T> {

    pub fn new(error: T, from: usize, to: usize) -> Self {
        Self {
            error,
            span: Span::new(from, to),
            message: None
        }
    }

    pub fn of(error: T, span: Span) -> Self {
        Self {
            error,
            span,
            message: None
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);

        self
    }
}