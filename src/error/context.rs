use std::cmp::{max, min};
use std::fmt::{Debug, Display};

pub type Errors<T> = Vec<ErrorContext<T>>;

#[derive(Debug, Clone)]
pub struct ErrorContext<T> {
    pub error: T,
    pub span_type: SpanType,
    pub message: Option<String>
}

#[derive(Debug, Copy, Clone)]
pub enum SpanType {
    SEGMENT(Span),
    LINE(usize),
    UNKNOWN
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


impl <T> ErrorContext<T> {

    pub fn new(error: T, from: usize, to: usize) -> Self {
        Self {
            error,
            span_type: SpanType::SEGMENT(Span::new(from, to)),
            message: None
        }
    }

    pub fn line(error: T, line: usize) -> Self {
        Self {
            error,
            span_type: SpanType::LINE(line),
            message: None
        }
    }
    
    pub fn unknown(error: T) -> Self {
        Self {
            error,
            span_type: SpanType::UNKNOWN,
            message: None
        }
    }

    pub fn of(error: T, span: Span) -> Self {
        Self {
            error,
            span_type: SpanType::SEGMENT(span),
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
        match self.span_type {
            SpanType::SEGMENT(span) => Self::format_span_err(span, &self.error, str),
            SpanType::LINE(line) => format!("{} on line {line}", self.error),
            SpanType::UNKNOWN => format!("{}", self.error)
        }
    }

    fn format_span_err(span: Span, error: &T, str: Vec<char>) -> String {
        let mut from = span.from;
        let to = span.to;

        let mut line = String::new();
        let mut single_line = true;

        for i in from..to {
            let ch = str[i];

            line += &*ch.to_string();

            if ch == '\n' && i != (to -1){
                if line.chars().filter(|ch| !ch.is_whitespace()).collect::<Vec<char>>().len() == 0 {
                    line.clear();
                    from = i+1;
                } else {
                    single_line = false;
                }
            }
        }

        if !single_line {
            let start_line = str[0..from].iter().filter(|ch| **ch == '\n').count() + 1;
            let end_line = str[0..to].iter().filter(|ch| **ch == '\n').count() + 1;

            return format!("{} on lines {start_line}-{end_line}", error);
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
        let l1 = format!("{} on line {line_num}", error);

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