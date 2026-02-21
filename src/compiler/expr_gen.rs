use crate::ast::expression::TypedExpr;
use crate::compiler::codegen::BytecodeGen;
use crate::compiler::expr_gen::Operation::{Load, Store};
use crate::lexer::token::SimpleToken;

#[derive(Copy, Clone)]
pub enum Operation {
    Load, // loads the expression on top of the stack
    Store // stores value on top of the stack into the expression
}
impl Operation {

    pub fn is_load(&self) -> bool {
        match self {
            Load => true,
            Store => false
        }
    }

    pub fn is_store(&self) -> bool {
        match self {
            Load => false,
            Store => true
        }
    }

}

impl TypedExpr {

    pub fn generate_bytecode(&self, generator: &mut BytecodeGen, operation: Operation) {
        match self {
            TypedExpr::BoolLiteralExpr(b) => {
                if operation.is_store() {
                    panic!("Invalid STORE for {self:?}")
                }

                if *b {
                    generator.i_const(1);
                } else {
                    generator.i_const(0);
                }
            }
            TypedExpr::IntLiteralExpr(i) => {
                if operation.is_store() {
                    panic!("Invalid STORE for {self:?}")
                }

                generator.i_const(*i);
            }
            TypedExpr::FloatLiteralExpr(f) => {
                if operation.is_store() {
                    panic!("Invalid STORE for {self:?}")
                }

                generator.f_const(*f)
            }
            TypedExpr::StringLiteralExpr(_) => {}
            TypedExpr::IdentifierExpr(t, name) => {
                match operation {
                    Load => {
                        generator.i_load(name.clone());
                    }
                    Store => {
                        generator.i_store(name.clone())
                    }
                }

            }
            TypedExpr::IncrementExpr(_, e) => {
                if operation.is_store() {
                    panic!("Store not yet implement for {self:?}")
                }

                e.generate_bytecode(generator, Load);

                generator.i_const(1);
                generator.add();

                e.generate_bytecode(generator, Store);
            }
            TypedExpr::DecrementExpr(_, e) => {
                if operation.is_store() {
                    panic!("Store not yet implement for {self:?}")
                }

                e.generate_bytecode(generator, Load);

                generator.i_const(1);
                generator.sub();

                e.generate_bytecode(generator, Store);
            }
            TypedExpr::PrefixExpr { .. } => {}
            TypedExpr::BinaryExpr {left, operator, right, .. } => {
                if operation.is_store() {
                    panic!("Invalid STORE for {self:?}")
                }

                left.generate_bytecode(generator, Load);
                right.generate_bytecode(generator, Load);

                match operator {
                    SimpleToken::Plus => generator.add(),
                    SimpleToken::Dash => generator.sub(),
                    SimpleToken::Star => generator.mul(),
                    SimpleToken::Slash => generator.div(),
                    SimpleToken::Percent => generator.modulo(),
                    _ => panic!("Invalid operator {:?}", operator)
                }
            }
            TypedExpr::AssignExpr {assignee, value, .. } => {
                value.generate_bytecode(generator, Load);
                assignee.generate_bytecode(generator, Store);
            }
            TypedExpr::TupleExpr { values, .. } => {
                for x in values {
                    x.generate_bytecode(generator, operation);
                }
            }
            TypedExpr::MemberExpr { member, property, .. } => {
                todo!()
            }
            TypedExpr::CallExpr { .. } => {
                todo!()
            }
        }
    }

}