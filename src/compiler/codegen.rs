use std::collections::HashMap;
use crate::analysis::symbol_table::SymbolTable;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::expression::Expression;
use crate::compiler::bytecode::ByteCode;
use crate::compiler::bytecode::ByteCode::{ALoad, ALoadOffset, ALoadVarOffset, AStore, AStoreOffset, AStoreVarOffset, Add, And, Call, CmpEq, CmpEqGreater, CmpEqLess, CmpGreater, CmpLess, CmpNotEq, Comment, CreateStackPtr, Div, Dup, FConst, FLoad, FLoadOffset, FLoadVarOffset, FStore, FStoreOffset, FStoreVarOffset, Goto, HeapAlloc, IConst, ILoad, ILoadOffset, ILoadVarOffset, IStore, IStoreOffset, IStoreVarOffset, IfEq, IfNe, Mod, Mul, Nop, NullPtr, Or, Pop, Ret, RetLong, StackFrame, Sub, Swap};

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
    scopes: Vec<ScopeInfo>,
    local_count: u16,
    scope_local_count: u16,
    symbol_table: SymbolTable<u16, ()>,
    pub functions: HashMap<String, FunctionInfo>,
    func_count: u16
}

impl BytecodeGen {

    pub fn new() -> Self {
        Self {
            instructions: vec![],
            scopes: vec![],
            local_count: 0,
            scope_local_count: 0,
            symbol_table: SymbolTable::new(),
            functions: HashMap::new(),
            func_count: 0
        }
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

    pub fn end_scope(&mut self) {
        let scope = self.scopes.pop().unwrap();

        for i in scope.exit_points {
            let end_offset = self.index() - i - 1;
            self.instructions[i as usize] = Goto(end_offset);
        }

        self.scope_local_count = scope.local_count;

        self.symbol_table.pop();
        if !scope.skippable {
            return;
        }

        let end_offset = self.index() - scope.start_index - 1;

        let ind = scope.start_index;

        match self.instructions[ind as usize] {
            IfEq(_) => self.instructions[ind as usize] = IfEq(end_offset),
            IfNe(_) => self.instructions[ind as usize] = IfNe(end_offset),
            
            _ => panic!("Expected comparison placeholder, got {:?}", self.instructions[ind as usize])
        }
    }

    pub fn get_scope_start_offset(&self) -> i32 {
        if self.scopes.is_empty() {
            panic!("UH OH! scopes are empty!")
        }

        let last = self.scopes.last().unwrap();

        last.start_index - self.index() - 1
    }

    pub fn fn_start(&mut self, name: String) {
        let fun = self.functions.get(&name).unwrap();
        self.functions.insert(name, FunctionInfo{
            index: fun.index,
            params_len: fun.params_len,
            start: (self.instructions.len() as u32),
            end: 0
        });

        self.new_scope();
        self.push_insn(StackFrame(0));
    }

    pub fn fn_end(&mut self, name: String) {
        let fun = self.functions.get(&name).unwrap();
        self.functions.insert(name, FunctionInfo{
            index: fun.index,
            params_len: fun.params_len,
            start: fun.start,
            end: (self.instructions.len() as u32)
        });

        let scope = self.scopes.last().unwrap();

        let frame_ind = scope.start_index;

        let local_count = self.local_count - scope.local_count;

        self.instructions[frame_ind as usize] = StackFrame(local_count);

        self.end_scope();

        self.local_count = self.scope_local_count;
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

    fn push_insn(&mut self, insn: ByteCode) {
        self.instructions.push(insn);
    }

    fn index(&self) -> i32 {
        self.instructions.len() as i32
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

    pub fn null_const(&mut self) {
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
    
    pub fn or(&mut self) {
        self.push_insn(Or);
    }
    
    pub fn and(&mut self) {
        self.push_insn(And);
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

    pub fn store_value(&mut self, registry: &TypeRegistry, name: &String, value: TypeEntry) {
        match value.get(registry) {
            AstType::Bool | AstType::Int => self.i_store(name.clone()),
            AstType::Float => self.f_store(name.clone()),
            AstType::NullableType { .. } => self.a_store(name.clone()),
            AstType::StructType { .. } => self.a_store(name.clone()),
            _ => panic!("Not yet supported type {:?}", value.get(registry))
        };
    }

    pub fn i_store(&mut self, name: String) {
        self.store(name, |i| IStore(i))
    }

    pub fn f_store(&mut self, name: String) {
        self.store(name, |i| FStore(i))
    }

    pub fn a_store(&mut self, name: String) {
        self.store(name, |i| AStore(i))
    }

    pub fn store_offset_value(&mut self, registry: &TypeRegistry, offset: u32, value: TypeEntry) {
        match value.get(registry) {
            AstType::Bool | AstType::Int => self.i_store_offset(offset),
            AstType::Float => self.f_store_offset(offset),
            AstType::NullableType { .. } => self.a_store_offset(offset),
            AstType::StructType { .. } => self.a_store_offset(offset),
            _ => panic!("Not yet supported type {:?}", value.get(registry))
        };
    }

    pub fn i_store_offset(&mut self, offset: u32) {
        self.push_insn(IStoreOffset(offset));
    }

    pub fn f_store_offset(&mut self, offset: u32) {
        self.push_insn(FStoreOffset(offset));
    }

    pub fn a_store_offset(&mut self, offset: u32) {
        self.push_insn(AStoreOffset(offset));
    }

    fn store(&mut self, name: String, create_store: impl Fn(u16) -> ByteCode) {
        if self.symbol_table.contains(&name) {
            let ind = self.symbol_table.get(name).unwrap();

            self.push_insn(create_store(ind));
        } else {
            panic!("Variable with the name {name} not created")
        }
    }
    
    pub fn store_new_var(&mut self, name: String, registry: &TypeRegistry, value: TypeEntry) {
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
                    self.store_internal_value(registry, *a);
                }
                self.scope_local_count += arr.len() as u16 - 1;

                self.create_stack_ptr(arr.len() as u16);

                self.store_new(name, |i| AStore(i))
            },
            _ => panic!("Not yet supported type {:?}", value.get(registry))
        };
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

    pub fn store_internal_value(&mut self, registry: &TypeRegistry, value: TypeEntry) {
        match value.get(registry) {
            AstType::Bool | AstType::Int => self.store_internal(|i| IStore(i)),
            AstType::Float => self.store_internal(|i| FStore(i)),
            AstType::NullableType { .. } => self.store_internal(|i| AStore(i)),
            AstType::StructType { .. } => self.store_internal(|i| AStore(i)),
            _ => panic!("Not yet supported type {:?}", value.get(registry))
        };
    }

    fn store_internal(&mut self, create_store: impl Fn(u16) -> ByteCode) {
        self.push_insn(create_store(self.scope_local_count));

        self.local_count += 1;
        self.scope_local_count += 1;
    }


    pub fn create_stack_ptr(&mut self, consume_words: u16) {
        let offset = self.local_count - consume_words;

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

    pub fn i_load_offset(&mut self, offset: u16) {
        self.push_insn(ILoadOffset(offset));
    }

    pub fn f_load_offset(&mut self, offset: u16) {
        self.push_insn(FLoadOffset(offset));
    }

    pub fn a_load_offset(&mut self, offset: u16) {
        self.push_insn(ALoadOffset(offset));
    }

    pub fn i_load_var_offset(&mut self) {
        self.push_insn(ILoadVarOffset);
    }

    pub fn f_load_var_offset(&mut self) {
        self.push_insn(FLoadVarOffset);
    }

    pub fn a_load_var_offset(&mut self) {
        self.push_insn(ALoadVarOffset);
    }

    pub fn i_store_var_offset(&mut self) {
        self.push_insn(IStoreVarOffset);
    }

    pub fn f_store_var_offset(&mut self) {
        self.push_insn(FStoreVarOffset);
    }

    pub fn a_store_var_offset(&mut self) {
        self.push_insn(AStoreVarOffset);
    }

    pub fn call(&mut self, name: &String) {
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