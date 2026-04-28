use crate::error::context::{ErrorContext, Span, SpanType};
use crate::error::runtime_error::RuntimeError;


#[derive(Debug)]
pub struct RuntimeErrorContext {
    context: ErrorContext<RuntimeError>,
    call_stack: Vec<(String, usize)>
}

impl RuntimeErrorContext {

    pub fn new(context: ErrorContext<RuntimeError>, call_stack: Vec<(String, usize)>) -> Self {
        Self {
            context, call_stack
        }
    }

    pub fn format(&self, str: Vec<char>) -> String {
        let mut res = self.context.format(str);

        res.push('\n');

        res.push_str("\nstack backtrace:\n");

        let mut index = 0;
        for (name, line) in &self.call_stack {
            let abc = format!("    {index}: {name}\n");
            let str = format!("        at line {line}\n");

            index += 1;

            res.push_str(abc.as_str());
            res.push_str(str.as_str());
        }

        res
    }

}