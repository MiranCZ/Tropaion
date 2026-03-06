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
        Debug::fmt(&self.from, f)?;
        Debug::fmt("-", f)?;
        Debug::fmt(&self.to, f)
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

impl <T: Display> ErrorContext<T> {

    pub fn format(&self, str: Vec<char>) -> String {
        let from = self.span.from;
        let to = self.span.to;

        let mut line = String::new();
        let mut single_line = true;

        for i in from..to {
            let ch = str[i];

            line += &*ch.to_string();

            if ch == '\n' {
                single_line = false;
            }
        }

        if !single_line {
            let start_line = str[0..from].iter().filter(|ch| **ch == '\n').count() + 1;
            let end_line = str[0..to].iter().filter(|ch| **ch == '\n').count() + 1;

            return format!("{} on lines {start_line}-{end_line}", self.error);
        }

        let mut before = String::new();
        for i in (0..from).rev() {
            let ch = str[i];
            if ch == '\n' {
                break;
            }
            before.push(' ');

            line = ch.to_string() + &*line;
        }

        for i in (to..str.len()) {
            let ch = str[i];
            if ch == '\n' {
                break;
            }

            line = line + &*ch.to_string();
        }

        let line_num = str[0..from].iter().filter(|ch| **ch == '\n').count() + 1;
        let l1 = format!("{} on line {line_num}", self.error);

        let mut can_ignore = true;
        for i in from..to {
            // don't point to empty leading space
            if str[i] == ' ' && can_ignore {
                before.push(' ');
                continue;
            } else {
                can_ignore = false;
            }

            before.push('^');
        }

        format!("{l1}\n{line}\n{before}\n")
    }

}