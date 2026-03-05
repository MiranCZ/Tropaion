use crate::error::context::Span;

#[macro_export]
macro_rules! spanned {
    ($parser:expr, $body:block) => {{
        let from = $parser.current_span().from;
        let inner = { $body };
        let to = $parser.current_span().to;
        Ok(crate::util::spanned::Spanned::new(inner, from, to))
    }};
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span
}

impl <T> Spanned<T> {
    
    pub fn of(node: T, span: Span) -> Self {
        Self {
            node, span
        }
    }
    
    pub fn new(node: T, from: usize, to: usize) -> Self {
        Self {
            node,
            span: Span::new(from, to)
        }
    }
    
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
    
}