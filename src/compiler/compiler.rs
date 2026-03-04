use std::collections::HashMap;
use crate::analysis::type_registry::TypeRegistry;
use crate::ast::statement::Statement::{BlockStmt, StructStmt};
use crate::ast::statement::{Statement, TypedStmt};
use crate::compiler::bytecode::ByteCode;
use crate::compiler::codegen::{BytecodeGen, FunctionInfo};
use crate::error::compilation_error::CompilationError;

pub struct Compiler {
    root: TypedStmt,
    pub generator: BytecodeGen
}


impl Compiler {
    pub fn new(root: TypedStmt) -> Self {
        Self {
            root,
            generator: BytecodeGen::new()
        }
    }

    pub fn compile(mut self, registry: &TypeRegistry) -> Result<(Vec<ByteCode>, HashMap<String, FunctionInfo>), CompilationError> {
        self.collect_functions(registry, &self.root.clone());

        self.root.gen_bytecode(registry, &mut self.generator)?;

        Ok((self.generator.instructions, self.generator.functions))
    }

    fn collect_functions(&mut self, registry: &TypeRegistry ,stmt: &TypedStmt) {
        match stmt {
            BlockStmt { body, .. } |
            TypedStmt::IfStmt { body, .. } |
            TypedStmt::WhileStmt { body, .. } |
            StructStmt { body, .. } => {
                for b in body {
                    self.collect_functions(registry,b)
                }
            }
            Statement::FunctionStmt {name,params ,..} => {
                let mut size = 0;

                for p in params {
                    size += p.param_type.get(registry).word_size(registry);
                }

                self.generator.register_func(name.clone(), size);
            }


            _ => {}
        }
    }
}