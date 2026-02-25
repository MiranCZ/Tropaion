use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Rem, Sub};
use crate::compiler::bytecode::ByteCode;
use crate::compiler::codegen::FunctionInfo;
use crate::interpreter::interpreter::Value::{FloatValue, IntValue, Null, RefValue};
use crate::interpreter::interpreter::ValueType::{Address, Float, Int};

macro_rules! impl_math_op {
    ($trait:ident, $method:ident) => {
        impl $trait for Value {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self::Output {
                match self {
                    Value::IntValue(v) => Value::IntValue(v.$method(rhs.try_as_int())),
                    Value::FloatValue(v) => Value::FloatValue(v.$method(rhs.try_as_float())),
                    Value::RefValue(_) => panic!("Cannot apply {} to references", stringify!($method)),
                    Value::Null => panic!("Cannot write operations with null!")
                }
            }
        }
    };
}

macro_rules! math_op {
    ($method:ident) => {
        fn $method(&mut self) {
            let a = self.pop();
            let b = self.pop();

            self.push(a.$method(b));
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Value {
    Null,
    IntValue(i32),
    FloatValue(f32),
    RefValue(u32)
}

#[derive(Debug, Clone, Copy)]
enum ValueType {
    Null,
    Int,
    Float,
    Address
}

impl ValueType {

    pub fn assignable(&self, value: &Value) -> bool {
        match (self, value) {
            (ValueType::Null, Null) |
            (Int, IntValue(_)) |
            (Float, FloatValue(_)) |
            (Address, RefValue(_)) => true,

            _ => false
        }
    }

}

impl Value {

    pub fn try_as_int(&self) -> i32 {
        if let IntValue(v) = self {
            return *v;
        }

        panic!("Attempting to get {self:?} as int!");
    }

    pub fn try_as_float(&self) -> f32 {
        if let FloatValue(v) = self {
            return *v;
        }

        panic!("Attempting to get {self:?} as float!");
    }

}

impl_math_op!(Add, add);
impl_math_op!(Sub, sub);
impl_math_op!(Mul, mul);
impl_math_op!(Div, div);
impl_math_op!(Rem, rem);

const STACK_SIZE: usize = 1_000_000;

pub struct Interpreter {
    instructions: Vec<ByteCode>,
    functions: Vec<FunctionInfo>,
    function_mapping: HashMap<String, FunctionInfo>,

    insn_addr: usize,
    pointer: usize,
    stack: Vec<Value>,
    stack_frames: Vec<usize>,
    call_stack: Vec<usize>
}

impl Interpreter {

    pub fn new(instructions: Vec<ByteCode>, functions_map: HashMap<String, FunctionInfo>) -> Self {
        let mut functions = Vec::with_capacity(functions_map.len());
        functions.resize(functions_map.len(),FunctionInfo{index: 0, start: 0, end: 0, params_len: 0});

        let mut start_addr = 0;

        for e in functions_map.iter() {
            let f = e.1;

            functions[f.index as usize] = *f;

            if e.0 == "main_" {
                start_addr = f.start;
            }
        }
        let mut stack = Vec::with_capacity(STACK_SIZE);
        stack.resize(STACK_SIZE, Null);

        Self {
            instructions, functions,
            function_mapping: functions_map,

            insn_addr: 0,
            pointer: 0,
            stack,
            stack_frames: vec![],
            call_stack: vec![]
        }
    }

    pub fn run_function(&mut self, function: String) -> Vec<Value> {
        let fun = self.function_mapping.get(&function);
        if fun.is_none() {
            panic!("Trying to call non-existant function {function}!");
        }
        let fun = fun.unwrap();

        self.insn_addr = self.instructions.len();
        self.call(fun.index);
        self.insn_addr += 1;

        while self.insn_addr < self.instructions.len() {
            let insn = self.instructions[self.insn_addr].clone();

            println!("\tVALUES: {:?}", &self.stack[0..self.pointer]);
            println!("Executing {insn:?}\n");

            self.execute(insn);

            self.insn_addr += 1;
        }

        let mut result = vec![];

        for v in self.stack[0..self.pointer].iter() {
            result.push(*v);
        }

        result
    }

    fn execute(&mut self, insn: ByteCode) {
        match insn {
            ByteCode::Comment(_) => {}
            ByteCode::Nop => {}

            ByteCode::IConst(i) => self.push_int(i),
            ByteCode::FConst(f) => self.push_float(f),
            ByteCode::Pop => {self.pop();},
            ByteCode::Dup => self.dup(),
            ByteCode::Add => self.add(),
            ByteCode::Sub => self.sub(),
            ByteCode::Mul => self.mul(),
            ByteCode::Div => self.div(),
            ByteCode::Mod => self.rem(),

            ByteCode::CmpEq => {}
            ByteCode::CmpNotEq => {}
            ByteCode::CmpGreater => {}
            ByteCode::CmpEqGreater => {}
            ByteCode::CmpLess => {}
            ByteCode::CmpEqLess => {}

            ByteCode::IStore(i) => self.store_local(i, Int),
            ByteCode::FStore(i) => self.store_local(i, Float),
            ByteCode::AStore(i) => self.store_local(i, Address),

            ByteCode::ILoad(i) => self.load_local(i, Int),
            ByteCode::FLoad(i) => self.load_local(i, Float),
            ByteCode::ALoad(i) => self.load_local(i, Address),

            ByteCode::CreateStackPtr { consume_words } => self.create_stack_ptr(consume_words),

            ByteCode::ILoadOffset(o) => self.load_offset_local(o, Int),
            ByteCode::FLoadOffset(o) => self.load_offset_local(o, Float),
            ByteCode::ALoadOffset(o) => self.load_offset_local(o, Address),

            ByteCode::IStoreOffset(o) => self.store_offset_local(o, Int),
            ByteCode::FStoreOffset(o) => self.store_offset_local(o, Float),
            ByteCode::AStoreOffset(o) => self.store_offset_local(o, Address),

            ByteCode::Goto(i) => self.goto(i),
            ByteCode::IfEq(o) => self.if_eq(o),
            ByteCode::IfNe(o) => self.if_ne(o),


            ByteCode::Call(ind) => self.call(ind),
            ByteCode::Ret(size) => self.ret(size as u32),
            ByteCode::RetLong(size) => self.ret(size),

            ByteCode::StackFrame(_) => panic!("Dangling stack frame instruction!"),
        }
    }

    fn push_int(&mut self, value: i32) {
        self.push(IntValue(value));
    }

    fn push_float(&mut self, value: f32) {
        self.push(FloatValue(value));
    }


    fn pop(&mut self) -> Value {
        if self.pointer == 0 {
            panic!("Attempting to pop an empty stack!")
        }

        self.pointer -= 1;

        self.stack[self.pointer]
    }

    fn peek(&mut self) -> Value {
        if self.pointer == 0 {
            panic!("Attempting to peek an empty stack!")
        }

        self.stack[self.pointer-1]
    }

    fn push(&mut self, value: Value) {
        self.stack[self.pointer] = value;

        self.pointer += 1;
    }

    fn push_stack_frame(&mut self, size: u16) {
        self.stack_frames.push(self.pointer);

        self.pointer += size as usize;
    }

    fn pop_stack_frame(&mut self) {
        let frame = self.stack_frames.pop();

        if let Some(value) = frame {
            self.pointer = value;
        } else {
            panic!("Attempted to pop stack frame but none is present!")
        }
    }

    fn store_local(&mut self, index: u16, typ: ValueType) {
        let top = self.pop();

        let absolute_index = self.get_stack_frame_start() + (index as usize);

        if !typ.assignable(&top) {
            panic!("Invalid store {typ:?} {top:?}")
        }

        self.stack[absolute_index] = top;
    }

    fn load_local(&mut self, index: u16, typ: ValueType) {
        let absolute_index = self.get_stack_frame_start() + (index as usize);

        let value = self.stack[absolute_index];


        if !typ.assignable(&value) {
            panic!("Invalid LOAD {typ:?} {value:?}")
        }

        self.push(value);
    }


    fn store_offset_local(&mut self, offset: u16, typ: ValueType) {
        let top = self.pop();

        if let RefValue(addr) = top {
            let absolute_index = (addr as usize) + (offset as usize);

            let prev = self.stack[absolute_index];


            let new = self.pop();

            if !typ.assignable(&prev) || !typ.assignable(&new) {
                panic!("Invalid STORE_OFFSET expected: {typ:?} previous: {prev:?} new: {new:?}");
            }

            self.stack[absolute_index] = new;
        }
    }

    fn load_offset_local(&mut self, offset: u16, typ: ValueType) {
        let top = self.pop();

        if let Value::RefValue(addr) = top {
            let absolute_index = (addr as usize) + (offset as usize);

            let value = self.stack[absolute_index];

            if !typ.assignable(&value) {
                panic!("Invalid LOAD_OFFSET {typ:?} {value:?}")
            }

            self.push(value);
        } else {
            panic!("Tried to call LOAD_OFFSET with {top:?}")
        }

    }

    fn create_stack_ptr(&mut self, size: u32) {
        let addr = (self.pointer as u32) - (size);

        self.push(RefValue(addr));
    }

    fn get_stack_frame_start(&self) -> usize {
        let value = self.stack_frames.last();

        if let Some(ind) = value {
            return *ind;
        }

        panic!("Attempted to get stack frame but none is present!")
    }

    fn dup(&mut self) {
        let last = self.peek();

        self.push(last)
    }

    math_op!(add);
    math_op!(sub);
    math_op!(mul);
    math_op!(div);
    math_op!(rem);

    fn call(&mut self, fn_index: u16) {
        let info = self.functions[fn_index as usize];

        self.call_stack.push(self.insn_addr);

        let size = (info.params_len) as usize;

        let mut values = Vec::with_capacity(size);

        for _ in 0..size {
            values.push(self.pop());
        }
        values.reverse();

        self.insn_addr = (info.start as usize);

        let next_insn = &self.instructions[self.insn_addr];

        if let ByteCode::StackFrame(size) = next_insn {
            self.push_stack_frame(*size);
        } else {
            panic!("Expected STACK_FRAME after CALL got {next_insn:?} instead")
        }

        for v in values {
            self.push(v);
        }

    }

    fn ret(&mut self, size: u32) {
        let size = size as usize;

        let mut values = Vec::with_capacity(size);

        for v in  self.stack[(self.pointer-size)..self.pointer].iter() {
            values.push(*v);
        }

        self.pop_stack_frame();

        for v in values {
            self.push(v);
        }

        let return_address = self.call_stack.pop();

        if let Some(addr) = return_address {
            self.insn_addr = addr;
        } else {
            panic!("Returned called on an empty callstack!")
        }

    }

    fn if_eq(&mut self, offset: i32) {
        let value = self.pop();

        let res = match value {
            IntValue(v) => v == 0,
            FloatValue(v) => v == 0f32,

            _ => panic!("Expected Number got {value:?}")
        };

        if res {
            self.goto(offset);
        }
    }

    fn if_ne(&mut self, offset: i32) {
        let value = self.pop();

        let res = match value {
            IntValue(v) => v == 0,
            FloatValue(v) => v == 0f32,

            _ => panic!("Expected Number got {value:?}")
        };

        if !res {
            self.goto(offset);
        }
    }

    fn goto(&mut self, offset: i32) {
        if (offset as usize) > self.insn_addr {
            panic!("Goto instruction pointer underflow!")
        }

        if (offset as usize) + self.insn_addr >= self.instructions.len() {
            panic!("Goto instruction pointer overflow!")
        }

        self.insn_addr += (offset as usize);
    }

}