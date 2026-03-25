use std::cell::Ref;
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};
use crate::ast::ast_type::AstType::Bool;
use crate::compiler::bytecode::ByteCode;
use crate::compiler::codegen::FunctionInfo;
use crate::compiler::compiler::CompilationResult;
use crate::error::context::ErrorContext;
use crate::error::ok;
use crate::error::runtime_error::{RuntimeError, ValueTypeVariant};
use crate::error::runtime_error::RuntimeError::{EmptyCallstack, FunctionNotFound, IllegalAllocSize, IllegalAssignment, InstructionPtrOverflow, InstructionPtrUnderflow, NullPtrDeref, StackFrameExpected, StackFrameMissing, StackOverflow, StackUnderflow, TypeMismatch, UnexpectedStackFrame};
use crate::error::runtime_error::ValueTypeVariant::Number;
use crate::interpreter::heap::Heap;
use crate::interpreter::value::Value;
use crate::interpreter::value::ValueType;
use crate::interpreter::value::Value::{FloatValue, IntValue, Null, RefValue};
use crate::interpreter::value::ValueType::{Address, Float, Int};
use crate::memory_blob::{resolve_blob, MemoryBlob};
use crate::util::arg_convertor::ValueConvertable;

macro_rules! math_op {
    ($method:ident) => {
        fn $method(&mut self) -> Res {
            let b = self.pop()?;
            let a = self.pop()?;

            self.push(a.$method(b))?;

            ok()
        }
    };
}

macro_rules! cmp_op {
    ($method:ident, $op: tt) => {
        fn $method(&mut self) -> Res {
            let b = self.pop()?;
            let a = self.pop()?;

            if a $op b {
                self.push(IntValue(1))?;
            } else {
                self.push(IntValue(0))?;
            }

            ok()
        }
    };
}


const STACK_SIZE: usize = 1_000_000;

struct StackFrame {
    start: usize,
    len: usize
}

pub struct Interpreter {
    instructions: Vec<ByteCode>,
    line_maps: Vec<usize>,
    functions: Vec<FunctionInfo>,
    function_mapping: HashMap<String, FunctionInfo>,

    insn_addr: usize,
    pointer: usize,
    stack: Vec<Value>,
    stack_frames: Vec<StackFrame>,
    call_stack: Vec<usize>,
    heap: Heap
}

type Res = Result<(), RuntimeError>;
type ValueRes = Result<Value, RuntimeError>;


impl Interpreter {


    pub fn new(compilation_result: CompilationResult) -> Self {
        let mut functions = Vec::with_capacity(compilation_result.functions.len());
        functions.resize(compilation_result.functions.len(),FunctionInfo{index: 0, start: 0, end: 0, params_len: 0});


        for e in compilation_result.functions.iter() {
            let f = e.1;

            functions[f.index as usize] = *f;
        }

        let mut stack = Vec::with_capacity(STACK_SIZE);
        stack.resize(STACK_SIZE, Null);

        Self {
            instructions: compilation_result.instructions,
            line_maps: compilation_result.lines,
            function_mapping: compilation_result.functions,
            functions,

            insn_addr: 0,
            pointer: 1, // 0 is nullptr
            stack,
            stack_frames: vec![],
            call_stack: vec![],
            heap: Heap::new(STACK_SIZE)
        }
    }


    pub fn run_function(&mut self, function: String, arguments: Vec<ValueConvertable>) -> Result<MemoryBlob, ErrorContext<RuntimeError>> {
        let res = self._run_function(function, arguments);

        if let Ok(v) = res {
            return Ok(v);
        } else if let Err(e) = res {
            let line_num = self.line_maps[min(self.insn_addr, self.line_maps.len()-1)];

            return Err(ErrorContext::line(e, line_num));
        }

        panic!()
    }

    fn _run_function(&mut self, function: String, arguments: Vec<ValueConvertable>) -> Result<MemoryBlob, RuntimeError> {
        for a in arguments {
            for value in a.into_value(self) {
                self.push(value)?;
            }
        }

        let fun = self.function_mapping.get(&function);
        let fun = if let Some(fun) = fun {
            fun
        } else {
            return Err(FunctionNotFound(function));
        };

        self.insn_addr = self.instructions.len();
        self.call(fun.index)?;
        self.insn_addr += 1;

        while self.insn_addr < self.instructions.len() {
            let insn = self.instructions[self.insn_addr].clone();

            // println!("values {:?} {:?}", &self.stack[0..self.pointer], self.heap);
            // println!("\t{insn:?}\n");

            self.execute(insn)?;
            self.insn_addr += 1;
        }

        for v in self.stack[0..self.pointer].iter() {
            println!("VALUE: {v:?}");
        }

        Ok(resolve_blob(self.pop()?, self))
    }

    pub fn stack_top(&self) -> u32 {
        self.pointer as u32
    }
    pub unsafe fn get_heap(&mut self) -> &mut Heap {
        &mut self.heap
    }

    pub unsafe fn load_at_ptr(&self, ptr: usize) -> Value {
        self._load_at_ptr(ptr)
    }

    fn execute(&mut self, insn: ByteCode) -> Result<(), RuntimeError> {
        match insn {
            ByteCode::Comment(_) => {ok()}
            ByteCode::Nop => {ok()}

            ByteCode::NullPtr => self.push(RefValue {ptr: 0, len: 1}),

            ByteCode::IConst(i) => self.push_int(i),
            ByteCode::FConst(f) => self.push_float(f),
            ByteCode::Pop => {
                let v = self.pop();
                if let Some(e) = v.err() {
                   return Err(e);
                }

                ok()
            },
            ByteCode::Dup => self.dup(),
            ByteCode::Swap => self.swap(),
            
            ByteCode::I2F => self.i2f(),
            ByteCode::F2I => self.f2i(),

            ByteCode::Or => self.bitor(),
            ByteCode::And => self.bitand(),
            ByteCode::Xor => self.bitxor(),
            ByteCode::BitNot => self.not(),
            ByteCode::BoolNot => self.bool_not(),
            
            ByteCode::Shl => self.shl(),
            ByteCode::Shr => self.shr(),

            ByteCode::Add => self.add(),
            ByteCode::Sub => self.sub(),
            ByteCode::Mul => self.mul(),
            ByteCode::Div => self.div(),
            ByteCode::Mod => self.rem(),

            ByteCode::CmpEq => self.eq(),
            ByteCode::CmpNotEq => self.ne(),
            ByteCode::CmpGreater => self.gt(),
            ByteCode::CmpEqGreater => self.ge(),
            ByteCode::CmpLess => self.lt(),
            ByteCode::CmpEqLess => self.le(),

            ByteCode::Store(i) => self.store_local(i),

            ByteCode::Load(i) => self.load_local(i),

            ByteCode::CreateStackPtr { offset, consume_words } => self.create_stack_ptr(offset, consume_words),

            ByteCode::ILoadOffset(o) => self.load_offset_local(o),
            ByteCode::FLoadOffset(o) => self.load_offset_local(o),
            ByteCode::ALoadOffset(o) => self.load_offset_local(o),

            ByteCode::IStoreOffset(o) => self.store_offset_local(o),
            ByteCode::FStoreOffset(o) => self.store_offset_local(o),
            ByteCode::AStoreOffset(o) => self.store_offset_local(o),

            ByteCode::LoadVarOffset => self.load_var_offset_local(Int),

            ByteCode::StoreVarOffset => self.store_var_offset_local(Int),


            ByteCode::Goto(i) => self.goto(i),
            ByteCode::IfEq(o) => self.if_eq(o),
            ByteCode::IfNe(o) => self.if_ne(o),


            ByteCode::Call(ind) => self.call(ind),
            ByteCode::Ret(size) => self.ret(size as u32),
            ByteCode::RetLong(size) => self.ret(size),

            ByteCode::HeapAlloc(size) => self.heap_alloc(size),
            ByteCode::DynHeapAlloc => self.dyn_heap_alloc(),

            ByteCode::StackFrame(_) => Err(UnexpectedStackFrame),
        }
    }

    fn push_int(&mut self, value: i32) -> Res {
        self.push(IntValue(value))?;

        ok()
    }

    fn push_float(&mut self, value: f32) -> Res {
        self.push(FloatValue(value))?;

        ok()
    }


    fn pop(&mut self) -> ValueRes {
        if self.pointer == 0 {
            return Err(StackUnderflow("Attempting to pop"));
        }

        self.pointer -= 1;

        Ok(self.stack[self.pointer])
    }

    fn peek(&mut self) -> ValueRes {
        if self.pointer == 0 {
            return Err(StackUnderflow("Attempting to pop"));
        }

        Ok(self.stack[self.pointer-1])
    }

    fn push(&mut self, value: Value) -> Res {
        self.stack[self.pointer] = value;

        self.pointer += 1;
        if self.pointer >= self.stack.len() {
            return Err(StackOverflow)
        }

        ok()
    }

    fn push_stack_frame(&mut self, size: u16) {
        self.stack_frames.push(StackFrame{start: self.pointer, len: size as usize });

        self.pointer += size as usize;
    }

    fn pop_stack_frame(&mut self) -> Res {
        let frame = self.stack_frames.pop();

        if let Some(value) = frame {
            self.pointer = value.start;
        } else {
            return Err(StackFrameMissing);
        }

        ok()
    }

    fn store_local(&mut self, index: u16) -> Res {
        let top = self.pop()?;

        let absolute_index = self.get_stack_frame_start()? + (index as usize);

        self.stack[absolute_index] = top;

        ok()
    }

    fn load_local(&mut self, index: u16) -> Res {
        let absolute_index = self.get_stack_frame_start()? + (index as usize);

        let value = self.stack[absolute_index];

        self.push(value)?;

        ok()
    }

    fn store_var_offset_local(&mut self, typ: ValueType) -> Res {
        let top = self.pop()?;

        if let IntValue(o) = top {
            self.store_offset_local(o as u32)?;
        } else {
            return Err(TypeMismatch {expected: ValueTypeVariant::Int, got: top});
        }

        ok()
    }

    fn store_offset_local(&mut self, offset: u32) -> Res {
        let top = self.pop()?;

        if let RefValue{ptr, len} = top {
            if ptr == 0 {
                return Err(NullPtrDeref);
            }
            if len < (offset as u32) {
                return Err(RuntimeError::OffsetOutOfBounds {len, offset});
            }

            let absolute_index = (ptr as usize) + (offset as usize);

            let new = self.pop()?;

            self.store_at_ptr(absolute_index, new);
        } else {
            return Err(TypeMismatch {expected: ValueTypeVariant::Address, got: top});
        }

        ok()
    }

    fn load_var_offset_local(&mut self, typ: ValueType) -> Res{
        let top = self.pop()?;

        if let IntValue(o) = top {
            self.load_offset_local(o as u32)?;
        } else {
            return Err(TypeMismatch {expected: ValueTypeVariant::Int, got: top});
        }

        ok()
    }

    fn load_offset_local(&mut self, offset: u32) -> Res {
        let top = self.pop()?;

        if let RefValue{ptr, len} = top {
            // this is not a panic since LOADING a nullptr is valid for stuff like null-checks
            // if ptr == 0 {
            //     pa!("Loading from a null-pointer!");
            // }
            if len < (offset) {
                return Err(RuntimeError::OffsetOutOfBounds {len, offset});
            }

            let absolute_index = (ptr as usize) + (offset as usize);

            let value = self._load_at_ptr(absolute_index);

            self.push(value)?;
        } else {
            return Err(TypeMismatch {expected: ValueTypeVariant::Address, got: top});
        }

        ok()
    }

    fn _load_at_ptr(&self, ptr: usize) -> Value {
        if ptr < STACK_SIZE {
            self.stack[ptr]
        } else {
            self.heap.load(ptr as u32, 0)
        }
    }

    fn store_at_ptr(&mut self, ptr: usize, value: Value) {
        if ptr < STACK_SIZE {
            self.stack[ptr] = value;
        } else {
            self.heap.store(ptr as u32, 0, value);
        }
    }


    fn create_stack_ptr(&mut self, offset: u16, size: u16) -> Res {
        let ptr = self.get_stack_frame_start()? + (offset as usize);

        self.push(RefValue{ptr: ptr as u32, len: size as u32})?;

        ok()
    }

    fn get_stack_frame_start(&self) -> Result<usize, RuntimeError> {
        let value = self.stack_frames.last();

        if let Some(ind) = value {
            return Ok(ind.start);
        }

        Err(StackFrameMissing)
    }

    fn get_stack_frame_size(&self) -> Result<usize, RuntimeError> {
        let value = self.stack_frames.last();

        if let Some(ind) = value {
            return Ok(ind.len);
        }

        Err(StackFrameMissing)
    }

    fn dup(&mut self) -> Res {
        let last = self.peek()?;

        self.push(last)
    }

    fn swap(&mut self) -> Res {
        if self.stack.len() < 2 {
            return Err(StackUnderflow("Attempted to swap with small stack"));
        }

        self.stack.swap(self.pointer-2, self.pointer-1);

        ok()
    }
    
    fn i2f(&mut self) -> Res {
        let top = self.pop()?;

        if let IntValue(i) = top {
            self.push(FloatValue(i as f32))?;
        } else {
            return Err(TypeMismatch {expected: ValueTypeVariant::Int, got: top});
        }

        ok()       
    }

    fn f2i(&mut self) -> Res {
        let top = self.pop()?;

        if let FloatValue(f) = top {
            self.push(IntValue(f as i32))?;
        } else {
            return Err(TypeMismatch {expected: ValueTypeVariant::Float, got: top});
        }

        ok()
    }

    fn bool_not(&mut self) -> Res {
        let top = self.pop()?;
        
        if let IntValue(i) = top {
            if i == 0 {
                self.push(IntValue(1))?;
            } else {
                self.push(IntValue(0))?;
            }
        } else {
            return Err(TypeMismatch {expected: ValueTypeVariant::Int, got: top});
        }

        ok()
    }
    
    math_op!(bitor);
    math_op!(bitand);
    math_op!(bitxor);
    
    math_op!(shl);
    math_op!(shr);

    math_op!(add);
    math_op!(sub);
    math_op!(mul);
    math_op!(div);
    math_op!(rem);

    cmp_op!(gt, >);
    cmp_op!(ge, >=);
    cmp_op!(lt, <);
    cmp_op!(le, <=);

    fn not(&mut self) -> Res {
        let a = self.pop()?;
        
        self.push(a.not())?;
        
        ok()
    }
    
    fn eq(&mut self) -> Res {
        let b = self.pop()?;
        let a = self.pop()?;

        if self.values_equal(&a, &b, &mut HashSet::new()) {
            self.push(IntValue(1))?;
        } else {
            self.push(IntValue(0))?;
        }

        ok()
    }

    fn ne(&mut self) -> Res {
        let b = self.pop()?;
        let a = self.pop()?;

        if !self.values_equal(&a, &b, &mut HashSet::new()) {
            self.push(IntValue(1))?;
        } else {
            self.push(IntValue(0))?;
        }

        ok()
    }

    fn values_equal(&mut self, a: &Value, b: &Value, visited: &mut HashSet<(u32, u32)>) -> bool{
        let value = match (a, b) {
            (Null, Null) => true,
            (IntValue(i1), IntValue(i2)) => *i1 == *i2,
            (FloatValue(f1), FloatValue(f2)) => *f1 == *f2,
            (RefValue{ptr: p1, len: l1}, RefValue{ptr: p2, len: l2}) => 'block: {
                if *l1 != *l2 {
                    break 'block false;
                }
                if visited.contains(&(*p1, *p2)) {
                    break 'block true;
                }
                if *p1 == *p2 {
                    break 'block true;
                }

                visited.insert((*p1, *p2));
                for i in 0..(*l1) {
                    let v1 = self._load_at_ptr((i + p1) as usize);
                    let v2 = self._load_at_ptr((i + p2) as usize);

                    if !self.values_equal(&v1, &v2, visited) {
                        break 'block false;
                    }
                }

                true
            }

            _ => false,
        };

        if value {
            return true;
        }

        // deref of nulls etc
        match (a, b) {
            (RefValue {len, ..}, _) if *len == 1=> {
                if let Some(deref) = self.try_deref(*a, &mut HashSet::new()) {
                    self.values_equal(&deref, b, visited)
                } else {
                    false
                }
            }
            (_, RefValue {len, ..}) if *len == 1 => {
                if let Some(deref) = self.try_deref(*b, &mut HashSet::new()) {
                    self.values_equal(a, &deref, visited)
                } else {
                    false
                }
            }

            _ => false
        }
    }

    fn try_deref(&self, value: Value, visited: &mut HashSet<u32>) -> Option<Value> {
        if let RefValue {ptr, len} = value {
            if len != 1 {
                return Some(value);
            }

            if visited.contains(&ptr) {
                return None;
            }

            visited.insert(ptr);

            let at_ptr = self._load_at_ptr(ptr as usize);

            self.try_deref(at_ptr, visited)
        } else {
            Some(value)
        }
    }

    fn call(&mut self, fn_index: u16) -> Res {
        let info = self.functions[fn_index as usize];

        self.call_stack.push(self.insn_addr);

        let size = info.params_len as usize;

        let mut values = Vec::with_capacity(size);

        for _ in 0..size {
            values.push(self.pop()?);
        }
        values.reverse();

        println!("RESOLVED ARGS {:?} {size}", &self.stack[0..self.pointer]);
        self.insn_addr = (info.start as usize);

        let next_insn = &self.instructions[self.insn_addr];

        if let ByteCode::StackFrame(size) = next_insn {
            self.push_stack_frame(*size);
        } else {
            return Err(StackFrameExpected(next_insn.clone()));
        }

        for v in values {
            self.push(v)?;
        }

        ok()
    }

    fn ret(&mut self, size: u32) -> Res {
        let new_ptr = self.get_stack_frame_start()? as u32;

        let size = size as usize;

        let mut values = Vec::with_capacity(size);

        let mut promoted = HashMap::new();
        for v in self.stack[(self.pointer-size)..self.pointer].iter() {
            let mut v = *v;

            if let RefValue {ptr, len} = v && ptr >= new_ptr {
                if let Some(promoted_ptr) = promoted.get(&ptr) {
                    v = RefValue {ptr: *promoted_ptr, len};
                } else {
                    v = Self::promote_ref(ptr, len, new_ptr, &mut self.heap, &self.stack, &mut promoted);
                }
            }

            values.push(v);
        }

        self.pop_stack_frame()?;

        for v in values {
            self.push(v)?;
        }

        let return_address = self.call_stack.pop();

        if let Some(addr) = return_address {
            self.insn_addr = addr;
        } else {
            return Err(EmptyCallstack);
        }

        ok()
    }

    fn promote_ref(ptr: u32, len: u32, new_ptr: u32, heap: &mut Heap, stack: &Vec<Value>, promoted: &mut HashMap<u32, u32>) -> Value {
        if (ptr as usize) >= STACK_SIZE {
            return RefValue {ptr,len};
        }

        let mut values = Vec::with_capacity(len as usize);

        if let Some(p) = promoted.get(&ptr) {
            return RefValue {ptr: *p, len};
        }

        let promoted_ptr = heap.alloc(len);

        promoted.insert(ptr, promoted_ptr);


        for i in ptr..(ptr + len) {
            let i = i as usize;

            let mut v = stack[i];

            if let RefValue {ptr, len} = v && ptr >= new_ptr {
                v = Self::promote_ref(ptr, len, new_ptr, heap, stack, promoted);
            }

            values.push(v);
        }

        for i in 0..len {
            heap.store(promoted_ptr, i, values[i as usize]);
        }

        RefValue { ptr: promoted_ptr, len }
    }

    fn dyn_heap_alloc(&mut self) -> Res {
        let top = self.pop()?;

        if let IntValue(v) = top {
            if v < 0 {
                return Err(IllegalAllocSize(v));
            }

            self.heap_alloc(v as u32)
        } else {
            Err(TypeMismatch { expected: ValueTypeVariant::Int, got: top })
        }
    }
    
    fn heap_alloc(&mut self, size: u32) -> Res {
        let ptr =self.heap.alloc(size);

        self.push(RefValue {ptr, len: size})?;

        ok()
    }

    fn if_eq(&mut self, offset: i32) -> Res {
        let value = self.pop()?;

        let res = match value {
            IntValue(v) => v == 0,
            FloatValue(v) => v == 0f32,

            _ => return Err(TypeMismatch {expected: Number, got: value})
        };

        if res {
            self.goto(offset)?;
        }

        ok()
    }

    fn if_ne(&mut self, offset: i32) -> Res {
        let value = self.pop()?;

        let res = match value {
            IntValue(v) => v == 0,
            FloatValue(v) => v == 0f32,

            _ => return Err(TypeMismatch {expected: Number, got: value})
        };

        if !res {
            self.goto(offset)?;
        }

        ok()
    }

    fn goto(&mut self, offset: i32) -> Res {
        if offset < 0 {
            if (offset as i64) > (self.insn_addr as i64) {
                return Err(InstructionPtrUnderflow);
            }
        } else {
            if (offset as usize) + self.insn_addr >= self.instructions.len() {
                return Err(InstructionPtrOverflow);
            }
        }

        self.insn_addr = ((self.insn_addr as i64) + (offset as i64)) as usize;

        ok()
    }

}