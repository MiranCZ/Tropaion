use crate::compiler::compiler::CompilationResult;
use crate::interpreter::interpreter::Interpreter;

const DEFAULT_STACK_SIZE: usize = 10_000;
const DEFAULT_HEAP_SIZE: usize = 100_000;
const DEFAULT_MAX_INSTRUCTION_COST: usize = 1_000_000;

pub struct InterpreterBuilder {
    compilation_result: CompilationResult,

    stack_size: usize,
    heap_size: usize,
    max_instruction_cost: usize,
    profiling_enabled: bool,
}


impl InterpreterBuilder {

    pub fn new(compilation_result: CompilationResult) -> InterpreterBuilder {
        Self{
            compilation_result,

            stack_size: DEFAULT_STACK_SIZE,
            heap_size: DEFAULT_HEAP_SIZE,
            max_instruction_cost: DEFAULT_MAX_INSTRUCTION_COST,
            profiling_enabled: false,
        }
    }

    pub fn stack_size(mut self, stack_size: usize) -> InterpreterBuilder {
        self.stack_size = stack_size;
        self
    }

    pub fn heap_size(mut self, heap_size: usize) -> InterpreterBuilder {
        self.heap_size = heap_size;
        self
    }

    pub fn max_instruction_cost(mut self, max_instruction_cost: usize) -> InterpreterBuilder {
        self.max_instruction_cost = max_instruction_cost;
        self
    }

    pub fn with_profiling(mut self) -> InterpreterBuilder {
        self.profiling_enabled = true;
        self
    }

    pub fn build(self) -> Interpreter {
        Interpreter::new(self.compilation_result, self.stack_size, self.heap_size, self.max_instruction_cost, self.profiling_enabled)
    }

}