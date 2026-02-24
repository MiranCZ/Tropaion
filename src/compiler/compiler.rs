use crate::ast::ast_type::AstType;
use crate::ast::statement::Statement::{BlockStmt, StructStmt};
use crate::ast::statement::{Statement, TypedStmt};
use crate::compiler::codegen::BytecodeGen;

pub struct Compiler {
    root: TypedStmt,
    generator: BytecodeGen
}


impl Compiler {
    pub fn new(root: TypedStmt) -> Self {
        Self {
            root,
            generator: BytecodeGen::new()
        }
    }

    pub fn compile(&mut self) {
        self.collect_functions(&self.root.clone());

        self.root.gen_bytecode(&mut self.generator);

        println!("Table: {:?}",self.generator.functions);

        for i in self.generator.instructions.iter() {
            println!("{i:?}")
        }
    }

    fn collect_functions(&mut self, stmt: &TypedStmt) {
        match stmt {
            BlockStmt { body, .. } |
            TypedStmt::IfStmt { body, .. } |
            TypedStmt::WhileStmt { body, .. } |
            StructStmt { body, .. } => {
                for b in body {
                    self.collect_functions(b)
                }
            }
            Statement::FunctionStmt {name,params ,..} => {
                let mut size = 0;

                for p in params {
                    size += p.param_type.word_size();
                }

                self.generator.register_func(name.clone(), size);
            }


            _ => {}
        }
    }
}