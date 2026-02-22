use crate::ast::ast_type::AstType;
use crate::ast::statement::Statement::BlockStmt;
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
        self.collect_functions();

        self.root.gen_bytecode(&mut self.generator);

        for i in self.generator.instructions.iter() {
            println!("{i:?}")
        }
    }

    fn collect_functions(&mut self) {
        if let BlockStmt { body } = &self.root {
            for x in body {
                
                if let Statement::FunctionStmt {name, ..} = x {
                    self.generator.register_func(name.clone());
                }
            }
        }
    }
}