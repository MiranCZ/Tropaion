use std::collections::HashMap;
use crate::analysis::symbol_table::SymbolTable;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::compiler::bytecode::ByteCode;
use crate::compiler::bytecode::ByteCode::{ALoad, ALoadOffset, ALoadVarOffset, AStore, AStoreOffset, AStoreVarOffset, Add, And, BitNot, BoolNot, Call, CmpEq, CmpEqGreater, CmpEqLess, CmpGreater, CmpLess, CmpNotEq, Comment, CreateStackPtr, Div, Dup, FConst, FLoad, FLoadOffset, FLoadVarOffset, FStore, FStoreOffset, FStoreVarOffset, Goto, HeapAlloc, IConst, ILoad, ILoadOffset, ILoadVarOffset, IStore, IStoreOffset, IStoreVarOffset, IfEq, IfNe, Mod, Mul, Nop, NullPtr, Or, Pop, Ret, RetLong, Shl, Shr, StackFrame, Sub, Swap, Xor};
use crate::error::compilation_error::{CompilationError, EmptyRes};
use crate::error::compilation_error::CompilationError::{MissingScope, MissingVariable, UnsupportedType};
use crate::error::context::Span;
use crate::error::ok;
use crate::interpreter::value::ValueType;

#[derive(Debug)]
struct ScopeInfo {
    start_index: i32,
    skippable: bool,
    exit_points: Vec<i32>,
    local_count: u16
}

impl ScopeInfo {
    pub fn scope(bg: &BytecodeGen) -> Self {
        Self {
            start_index: bg.index(),
            skippable: false,
            exit_points: vec![],
            local_count: bg.local_count
        }
    }

    pub fn skippable(bg: &BytecodeGen) -> Self {
        Self {
            start_index: bg.index(),
            skippable: true,
            exit_points: vec![],
            local_count: bg.local_count
        }
    }

}

#[derive(Debug, Copy, Clone)]
pub struct FunctionInfo {
    pub index: u16,
    pub params_len: u32,
    pub start: u32,
    pub end: u32
}

#[derive(Debug)]
pub struct BytecodeGen {
    pub instructions: Vec<ByteCode>,
    pub lines: Vec<usize>,
    scopes: Vec<ScopeInfo>,
    line_stack: Vec<usize>,
    local_count: u16,
    scope_local_count: u16,
    to_box: Vec<(usize, TypeEntry)>,
    symbol_table: SymbolTable<u16, ()>,
    pub functions: HashMap<String, FunctionInfo>,
    func_count: u16,
    text: Vec<char>
}

impl BytecodeGen {

    pub fn new(text: Vec<char>) -> Self {
        Self {
            instructions: vec![],
            lines: vec![],
            scopes: vec![],
            to_box: vec![],
            line_stack: vec![],
            local_count: 0,
            scope_local_count: 0,
            symbol_table: SymbolTable::new(),
            functions: HashMap::new(),
            func_count: 0,
            text
        }
    }

    pub fn push_span(&mut self, span: Span) {
        let line = self.text[0..span.from].iter().filter(|ch| **ch == '\n').count() + 1;

        self.line_stack.push(line);
    }

    pub fn pop_span(&mut self) {
        self.line_stack.pop();
    }

    pub fn new_scope(&mut self) {
        self.scopes.push(ScopeInfo::scope(self));

        self.symbol_table.push();
    }

    pub fn new_skippable_scope_eq(&mut self) {
        self.scopes.push(ScopeInfo::skippable(self));
        self.symbol_table.push();

        // the zero is a placeholder
        self.push_insn(IfEq(0));
    }

    pub fn new_skippable_scope_ne(&mut self) {
        self.scopes.push(ScopeInfo::skippable(self));
        self.symbol_table.push();

        // the zero is a placeholder
        self.push_insn(IfNe(0));
    }

    pub fn push_scope_exit_insn(&mut self) {
        let ind = self.index();
        self.scopes.last_mut().unwrap().exit_points.push(ind);

        self.push_insn(Goto(0));
    }

    pub fn push_goto_scope_start_insn(&mut self) {
        let offset = self.scopes.last().unwrap().start_index - self.index() - 1;
        self.push_insn(Goto(offset))
    }

    pub fn end_scope(&mut self) -> EmptyRes {
        let scope = self.scopes.pop().unwrap();

        for i in scope.exit_points {
            let end_offset = self.index() - i - 1;
            self.instructions[i as usize] = Goto(end_offset);
        }

        self.scope_local_count = scope.local_count;

        self.symbol_table.pop();
        if !scope.skippable {
            return ok();
        }

        let end_offset = self.index() - scope.start_index - 1;

        let ind = scope.start_index;

        match self.instructions[ind as usize] {
            IfEq(_) => self.instructions[ind as usize] = IfEq(end_offset),
            IfNe(_) => self.instructions[ind as usize] = IfNe(end_offset),

            _ => return Err(CompilationError::ExpectedComparison(self.instructions[ind as usize].clone()))
        }

        ok()
    }

    pub fn get_scope_start_offset(&self) -> Result<i32, CompilationError> {
        if self.scopes.is_empty() {
            return Err(MissingScope);
        }

        let last = self.scopes.last().unwrap();

        Ok(last.start_index - self.index() - 1)
    }

    pub fn fn_start(&mut self, name: String) {
        assert!(self.to_box.is_empty());

        let fun = self.functions.get(&name).unwrap();
        self.functions.insert(name, FunctionInfo{
            index: fun.index,
            params_len: fun.params_len,
            start: self.instructions.len() as u32,
            end: 0
        });

        self.new_scope();
        self.push_insn(StackFrame(0));
    }

    pub fn fn_end(&mut self, name: String, registry: &TypeRegistry) -> EmptyRes {
        let fun = self.functions.get(&name).unwrap();
        self.functions.insert(name, FunctionInfo{
            index: fun.index,
            params_len: fun.params_len,
            start: fun.start,
            end: self.instructions.len() as u32
        });

        let scope = self.scopes.last().unwrap();

        let frame_ind = scope.start_index;

        let local_count = self.local_count - scope.local_count;

        let to_box = std::mem::take(&mut self.to_box);

        let tmp_count = to_box.len();

        let mut i = 0;
        let instructions = &mut self.instructions;

        for (pos, value) in to_box {
            Self::typed_expr(value, registry, |t| {
                let addr = local_count + i;
                i += 1;

                instructions[pos + 1] = CreateStackPtr {offset: addr, consume_words: 1};

                return match t {
                    ValueType::Null => Err(UnsupportedType("Null".to_string())),
                    ValueType::Int => Ok(instructions[pos] = IStore(addr)),
                    ValueType::Float => Ok(instructions[pos] = FStore(addr)),
                    ValueType::Address => Ok(instructions[pos] = AStore(addr))
                };
            }
            )?;
        }

        self.instructions[frame_ind as usize] = StackFrame(local_count+(tmp_count as u16));

        self.end_scope()?;

        self.local_count = self.scope_local_count;

        ok()
    }

    pub fn register_func(&mut self, name: String, params_len: u32) {
        self.functions.insert(name, FunctionInfo{
            index: self.func_count,
            params_len,
            start: 0,
            end: 0
        });
        self.func_count += 1;
    }

    pub fn pop_insn(&mut self) {
        self.instructions.pop();
        self.lines.pop();
    }

    fn push_insn(&mut self, insn: ByteCode) {
        let line = self.get_line();

        self.instructions.push(insn);
        self.lines.push(line);
    }

    fn get_line(&self) -> usize {
        if let Some(l) = self.line_stack.last() {
            return *l;
        }

        0
    }

    fn index(&self) -> i32 {
        self.instructions.len() as i32
    }

}

impl BytecodeGen {

    fn typed_expr(t: TypeEntry, registry: &TypeRegistry, mut generator: impl FnMut(ValueType) -> EmptyRes) -> EmptyRes {
        let t = t.get(registry);

        match t {
            AstType::Bool | AstType::Int => generator(ValueType::Int)?,
            AstType::Float => generator(ValueType::Float)?,
            AstType::NullableType { .. } |
            AstType::ArrayType {..} |
            AstType::StructType { .. } => generator(ValueType::Address)?,

            _ => return Err(CompilationError::unsupported_type(t, registry))
        }

        ok()
    }

    pub fn store_value(&mut self, registry: &TypeRegistry, name: &String, value: TypeEntry) -> EmptyRes {
        Self::typed_expr(value, registry, |t| match t {
            ValueType::Null => Err(UnsupportedType("Null".to_string())),
            ValueType::Int => self.store(name.clone(), |i| IStore(i)),
            ValueType::Float => self.store(name.clone(), |i| FStore(i)),
            ValueType::Address => self.store(name.clone(), |i| AStore(i)),
        })
    }

    pub fn store_offset_value(&mut self, registry: &TypeRegistry, offset: u32, value: TypeEntry) -> EmptyRes {
        Self::typed_expr(value, registry, |t| match t {
            ValueType::Null => Err(UnsupportedType("Null".to_string())),
            ValueType::Int => Ok(self.push_insn(IStoreOffset(offset))),
            ValueType::Float => Ok(self.push_insn(FStoreOffset(offset))),
            ValueType::Address => Ok(self.push_insn(AStoreOffset(offset)))
        })
    }


    pub fn store_internal_value(&mut self, registry: &TypeRegistry, value: TypeEntry) -> EmptyRes {
        Self::typed_expr(value, registry, |t| match t {
            ValueType::Null => Err(UnsupportedType("Null".to_string())),
            ValueType::Int => Ok(self.store_internal(|i| IStore(i))),
            ValueType::Float => Ok(self.store_internal(|i| FStore(i))),
            ValueType::Address => Ok(self.store_internal(|i| AStore(i))),
        })
    }

    pub fn store_boxed_value(&mut self, registry: &TypeRegistry, value: TypeEntry) -> EmptyRes {
        self.to_box.push((self.instructions.len(),value));
        self.nop(); // store
        self.nop(); // stack ptr

        ok()
    }

    pub fn store_var_offset(&mut self, registry: &TypeRegistry, value: TypeEntry) -> EmptyRes {
        Self::typed_expr(value, registry, |t| match t {
            ValueType::Null => Err(UnsupportedType("Null".to_string())),
            ValueType::Int => Ok(self.push_insn(IStoreVarOffset)),
            ValueType::Float => Ok(self.push_insn(FStoreVarOffset)),
            ValueType::Address => Ok(self.push_insn(AStoreVarOffset))
        })
    }

    pub fn load_offset_value(&mut self, registry: &TypeRegistry, offset: u32, value: TypeEntry) -> EmptyRes {
        Self::typed_expr(value, registry, |t| match t {
            ValueType::Null => Err(UnsupportedType("Null".to_string())),
            ValueType::Int => Ok(self.push_insn(ILoadOffset(offset))),
            ValueType::Float => Ok(self.push_insn(FLoadOffset(offset))),
            ValueType::Address => Ok(self.push_insn(ALoadOffset(offset)))
        })
    }

    pub fn load_var_offset(&mut self, registry: &TypeRegistry, value: TypeEntry) -> EmptyRes {
        Self::typed_expr(value, registry, |t| match t {
            ValueType::Null => Err(UnsupportedType("Null".to_string())),
            ValueType::Int => Ok(self.push_insn(ILoadVarOffset)),
            ValueType::Float => Ok(self.push_insn(FLoadVarOffset)),
            ValueType::Address => Ok(self.push_insn(ALoadVarOffset))
        })
    }

}


/// Instruction helpers
impl BytecodeGen {

    pub fn comment(&mut self, str: String) {
        self.push_insn(Comment(str));
    }

    pub fn nop(&mut self) {
        self.push_insn(Nop);
    }

    pub fn null_ptr(&mut self) {
        self.push_insn(NullPtr);
    }

    pub fn swap(&mut self) {
        self.push_insn(Swap);
    }

    pub fn i_const(&mut self, c: i32) {
        self.push_insn(IConst(c));
    }

    pub fn f_const(&mut self, c: f32) {
        self.push_insn(FConst(c));
    }

    pub fn pop(&mut self) {
        self.push_insn(Pop);
    }

    pub fn dup(&mut self) {
        self.push_insn(Dup);
    }
   
    pub fn bool_not(&mut self) {
        self.push_insn(BoolNot);
    }

    pub fn bit_not(&mut self) {
        self.push_insn(BitNot);
    }
    
    pub fn or(&mut self) {
        self.push_insn(Or);
    }
    
    pub fn and(&mut self) {
        self.push_insn(And);
    }

    pub fn xor(&mut self) {
        self.push_insn(Xor);
    }

    pub fn shl(&mut self) {
        self.push_insn(Shl);
    }

    pub fn shr(&mut self) {
        self.push_insn(Shr);
    }

    pub fn add(&mut self) {
        self.push_insn(Add);
    }

    pub fn sub(&mut self) {
        self.push_insn(Sub);
    }

    pub fn mul(&mut self) {
        self.push_insn(Mul);
    }

    pub fn div(&mut self) {
        self.push_insn(Div);
    }

    pub fn modulo(&mut self) {
        self.push_insn(Mod);
    }

    pub fn cmp_eq(&mut self) {
        self.push_insn(CmpEq);
    }

    pub fn cmp_ne(&mut self) {
        self.push_insn(CmpNotEq);
    }

    pub fn cmp_gt(&mut self) {
        self.push_insn(CmpGreater);
    }

    pub fn cmp_ge(&mut self) {
        self.push_insn(CmpEqGreater);
    }

    pub fn cmp_lt(&mut self) {
        self.push_insn(CmpLess);
    }

    pub fn cmp_le(&mut self) {
        self.push_insn(CmpEqLess);
    }

    fn store(&mut self, name: String, create_store: impl Fn(u16) -> ByteCode) -> EmptyRes {
        if self.symbol_table.contains(&name) {
            let ind = self.symbol_table.get(name).unwrap();

            self.push_insn(create_store(ind));
        } else {
            return Err(MissingVariable(name));
        }

        ok()
    }
    
    pub fn store_new_var(&mut self, name: String, registry: &TypeRegistry, value: TypeEntry) -> EmptyRes {
        match value.get(registry) {
            AstType::Bool | AstType::Int => self.store_new(name, |i| IStore(i)),
            AstType::Float => self.store_new(name, |i| FStore(i)),
            AstType::NullableType { .. } => self.store_new(name, |i| AStore(i)),
            AstType::StructType { .. } => self.store_new(name, |i| AStore(i)),
            AstType::ArrayType { .. } => self.store_new(name, |i| AStore(i)),
            AstType::TupleType(arr) => {
                self.scope_local_count += (arr.len() as u16) + 1;

                for a in arr.iter().rev() {
                    self.scope_local_count -= 2;
                    self.store_internal_value(registry, *a)?;
                }
                self.scope_local_count += arr.len() as u16 - 1;

                self.create_stack_ptr(arr.len() as u16);

                self.store_new(name, |i| AStore(i))
            },

            _ => return Err(CompilationError::unsupported_type(value.get(registry), registry))
        };

        ok()
    }
    
    fn store_new(&mut self, name: String, create_store: impl Fn(u16) -> ByteCode) {
        if self.symbol_table.contains_in_current(&name) {
            let ind = self.symbol_table.get(name).unwrap();

            self.push_insn(create_store(ind));
            return;
        }
        
        self.push_insn(create_store(self.scope_local_count));
        let ind = self.scope_local_count;

        self.local_count += 1;
        self.scope_local_count += 1;

        self.symbol_table.record(name, ind);
    }

    fn store_internal(&mut self, create_store: impl Fn(u16) -> ByteCode) {
        self.push_insn(create_store(self.scope_local_count));

        self.local_count += 1;
        self.scope_local_count += 1;
    }


    pub fn create_stack_ptr(&mut self, consume_words: u16) {
        let offset = self.scope_local_count - consume_words;

        self.push_insn(CreateStackPtr {offset, consume_words});
    }

    pub fn i_load(&mut self, name: String) {
        let ind = self.symbol_table.get(name).unwrap();

        self.push_insn(ILoad(ind));
    }

    pub fn f_load(&mut self, name: String) {
        let ind = self.symbol_table.get(name).unwrap();

        self.push_insn(FLoad(ind));
    }

    pub fn a_load(&mut self, name: String) {
        let ind = self.symbol_table.get(name).unwrap();

        self.push_insn(ALoad(ind));
    }

    pub fn call(&mut self, name: &String) {
        println!("Calling {name}, {:?}", self.functions);
        let info = self.functions.get(name).unwrap();

        self.push_insn(Call(info.index));
    }

    pub fn ret(&mut self, amount: u32) {
        if amount < (u16::MAX as u32) {
            self.push_insn(Ret(amount as u16));
        } else {
            self.push_insn(RetLong(amount));
        }
    }

    pub fn heap_alloc(&mut self, size: u32) {
        self.push_insn(HeapAlloc(size));
    }

}