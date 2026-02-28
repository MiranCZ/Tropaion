use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Rem, Sub};
use crate::compiler::bytecode::ByteCode;
use crate::compiler::codegen::FunctionInfo;
use crate::interpreter::heap::Heap;
use crate::interpreter::value::Value;
use crate::interpreter::value::ValueType;
use crate::interpreter::value::Value::{FloatValue, IntValue, Null, RefValue};
use crate::interpreter::value::ValueType::{Address, Float, Int};



macro_rules! math_op {
    ($method:ident) => {
        fn $method(&mut self) {
            let b = self.pop();
            let a = self.pop();

            self.push(a.$method(b));
        }
    };
}

macro_rules! cmp_op {
    ($method:ident, $op: tt) => {
        fn $method(&mut self) {
            let b = self.pop();
            let a = self.pop();

            if a $op b {
                self.push(IntValue(1));
            } else {
                self.push(IntValue(0));
            }
        }
    };
}


const STACK_SIZE: usize = 1_000_000;

pub struct Interpreter {
    instructions: Vec<ByteCode>,
    functions: Vec<FunctionInfo>,
    function_mapping: HashMap<String, FunctionInfo>,

    insn_addr: usize,
    pointer: usize,
    stack: Vec<Value>,
    stack_frames: Vec<usize>,
    call_stack: Vec<usize>,
    heap: Heap
}

impl Interpreter {

    pub fn new(instructions: Vec<ByteCode>, functions_map: HashMap<String, FunctionInfo>) -> Self {
        let mut functions = Vec::with_capacity(functions_map.len());
        functions.resize(functions_map.len(),FunctionInfo{index: 0, start: 0, end: 0, params_len: 0});


        for e in functions_map.iter() {
            let f = e.1;

            functions[f.index as usize] = *f;
        }

        let mut stack = Vec::with_capacity(STACK_SIZE);
        stack.resize(STACK_SIZE, Null);

        Self {
            instructions, functions,
            function_mapping: functions_map,

            insn_addr: 0,
            pointer: 1, // 0 is nullptr
            stack,
            stack_frames: vec![],
            call_stack: vec![],
            heap: Heap::new(STACK_SIZE)
        }
    }

    pub fn run_function(&mut self, function: String) -> (Vec<Value>, &Heap) {
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

            self.execute(insn);
            self.insn_addr += 1;
        }

        let mut result = vec![];

        for v in self.stack[0..self.pointer].iter() {
            result.push(*v);
        }

        (result, &self.heap)
    }

    fn execute(&mut self, insn: ByteCode) {
        match insn {
            ByteCode::Comment(_) => {}
            ByteCode::Nop => {}

            ByteCode::NullPtr => self.push(RefValue {ptr: 0, len: 1}),

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
            ByteCode::CmpGreater => self.gt(),
            ByteCode::CmpEqGreater => self.ge(),
            ByteCode::CmpLess => self.lt(),
            ByteCode::CmpEqLess => self.le(),

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

        if let RefValue{ptr, len} = top {
            if ptr == 0 {
                panic!("Storing to a null-pointer!");
            }
            if len < (offset as u32) {
                panic!("Reference offest is bigger than its length!")
            }

            let absolute_index = (ptr as usize) + (offset as usize);

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

        if let RefValue{ptr, len} = top {
            if ptr == 0 {
                panic!("Loading from a null-pointer!");
            }
            if len < (offset as u32) {
                panic!("Reference offest is bigger than its length!")
            }

            let absolute_index = (ptr as usize) + (offset as usize);

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
        let ptr = (self.pointer as u32) - (size);

        self.push(RefValue{ptr, len: size});
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

    cmp_op!(gt, >);
    cmp_op!(ge, >=);
    cmp_op!(lt, <);
    cmp_op!(le, <=);

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
        let new_ptr = self.get_stack_frame_start() as u32;

        let size = size as usize;

        let mut values = Vec::with_capacity(size);


        for v in self.stack[(self.pointer-size)..self.pointer].iter() {
            let mut v = *v;

            if let RefValue {ptr, len} = v && ptr > new_ptr {
                v = Self::promote_ref(ptr, len, new_ptr, &mut self.heap, &self.stack);
            }

            values.push(v);
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

    fn promote_ref(ptr: u32, len: u32, new_ptr: u32, heap: &mut Heap, stack: &Vec<Value>) -> Value {
        let mut values = Vec::with_capacity(len as usize);

        for i in ptr..(ptr + len) {
            let i = i as usize;

            let mut v = stack[i];

            if let RefValue {ptr, len} = v && ptr > new_ptr {
                v = Self::promote_ref(ptr, len, new_ptr, heap, stack);
            }

            values.push(v);
        }

        let ptr = heap.alloc(len);

        for i in 0..len {
            heap.store(ptr, i, values[i as usize]);
        }

        RefValue { ptr, len }
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
        if offset < 0 {
            if (offset as i64) > (self.insn_addr as i64) {
                panic!("Goto instruction pointer underflow!")
            }
        } else {
            if (offset as usize) + self.insn_addr >= self.instructions.len() {
                panic!("Goto instruction pointer overflow!")
            }
        }

        self.insn_addr = ((self.insn_addr as i64) + (offset as i64)) as usize;
    }

}