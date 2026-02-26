use std::collections::HashMap;
use crate::ast::ast_type::{AstType, MemberInfo};
use crate::ast::expression::{call, member, TypedExpr};
use crate::compiler::codegen::BytecodeGen;
use crate::compiler::expr_gen::Operation::{Load, LoadField, Store, StoreField};
use crate::lexer::token::SimpleToken;

#[derive(Clone)]
pub enum Operation {
    Load, // loads the expression on top of the stack
    Store, // stores value on top of the stack into the expression
    LoadField {
        fields: Vec<MemberInfo>,
        // fields and methods
        children: HashMap<String, MemberInfo>,
    },
    StoreField {
        fields: Vec<MemberInfo>
    },
}
impl TypedExpr {


    pub fn generate_bytecode(&self, generator: &mut BytecodeGen, operation: Operation) {
        match operation {
            Load => self.load(generator),
            Store => self.store(generator),
            LoadField {fields, children} => {
                self.load_field(fields, children, generator);
            }
            StoreField {fields} => {
                self.store_field(fields, generator);
            }
        }
    }

    pub fn load(&self, generator: &mut BytecodeGen) {
        match self {
            TypedExpr::BoolLiteralExpr(b) => {
                if *b {
                    generator.i_const(1);
                } else {
                    generator.i_const(0);
                }
            }
            TypedExpr::IntLiteralExpr(i) => generator.i_const(*i),
            TypedExpr::FloatLiteralExpr(f) => generator.f_const(*f),
            TypedExpr::StringLiteralExpr(_) => {
                todo!()
            }
            TypedExpr::IdentifierExpr(t, name) => {
                match t {
                    AstType::Bool |
                    AstType::Int => generator.i_load(name.clone()),
                    AstType::Float => generator.f_load(name.clone()),
                    AstType::StructType { .. } => generator.a_load(name.clone()),

                    _ => panic!("Invalid load type! {self:?}")
                }
            }
            TypedExpr::IncrementExpr(_, e) => {
                e.generate_bytecode(generator, Load);

                generator.i_const(1);
                generator.add();

                e.generate_bytecode(generator, Store);
            }
            TypedExpr::DecrementExpr(_, e) => {
                e.generate_bytecode(generator, Load);

                generator.i_const(1);
                generator.sub();

                e.generate_bytecode(generator, Store);
            }
            TypedExpr::PrefixExpr { operator, expr, .. } => {
                match operator {
                    SimpleToken::Dash => {
                        generator.i_const(0);
                        expr.generate_bytecode(generator, Load);
                        generator.sub();
                    },
                    SimpleToken::Tilde => todo!(),
                    SimpleToken::Exclamation => todo!(),

                    _ => panic!("Invalid operator {operator:?}")
                }
            }
            TypedExpr::BinaryExpr {left, operator, right, .. } => {
                left.generate_bytecode(generator, Load);
                right.generate_bytecode(generator, Load);

                match operator {
                    SimpleToken::Plus => generator.add(),
                    SimpleToken::Dash => generator.sub(),
                    SimpleToken::Star => generator.mul(),
                    SimpleToken::Slash => generator.div(),
                    SimpleToken::Percent => generator.modulo(),
                   
                    SimpleToken::Equals => generator.cmp_eq(),
                    SimpleToken::NotEquals => generator.cmp_ne(),
                    SimpleToken::Greater => generator.cmp_gt(),
                    SimpleToken::GreaterEquals => generator.cmp_ge(),
                    SimpleToken::Less => generator.cmp_lt(),
                    SimpleToken::LessEquals => generator.cmp_le(),
                    
                    _ => panic!("Invalid operator {:?}", operator)
                }
            }
            TypedExpr::AssignExpr {assignee, value, .. } => {
                value.generate_bytecode(generator, Load);
                assignee.generate_bytecode(generator, Store);
            }
            TypedExpr::TupleExpr { values, .. } => {
                for x in values {
                    x.generate_bytecode(generator, Load);
                }
            }
            TypedExpr::MemberExpr { member, property, .. } => {
                if let AstType::StructType {fields, children, ..} = member.get_type() {
                    member.generate_bytecode(generator, Load);

                    property.generate_bytecode(generator, LoadField {
                         fields, children
                    });
                } else {
                    panic!("Invalid member access for {:?}", member.get_type())
                }
            }
            TypedExpr::CallExpr {func, args, .. } => {
                let t = func.get_type();

                let mut size = 0;
                for a in args {
                    a.generate_bytecode(generator, Load);
                    size += a.get_type().word_size();
                }


                match t {
                    AstType::FunctionType {name, .. } => {
                        generator.call(&name);
                    }
                    AstType::StructType {name, fields, ..} => {
                        generator.create_stack_ptr(size);
                    }
                    _ => panic!("Cannot call {t:?}")
                }
            },
            _ => panic!("Invalid LOAD for {self:?}")
        }
    }

    pub fn store(&self, generator: &mut BytecodeGen) {
        match self {
            TypedExpr::IdentifierExpr(_, name) => generator.i_store(name.clone()),
            TypedExpr::IncrementExpr(_, e) => {
                todo!()
            }
            TypedExpr::DecrementExpr(_, e) => {
                todo!()
            }
            TypedExpr::PrefixExpr { .. } => {
                todo!()
            }
            TypedExpr::TupleExpr { values, .. } => {
                panic!("Tuples are immutable!");
            }
            TypedExpr::MemberExpr { member, property, .. } => {
                if let AstType::StructType {fields, ..} = member.get_type() {
                    member.generate_bytecode(generator, Load);

                    property.generate_bytecode(generator, StoreField {
                        fields
                    });
                } else {
                    panic!("Invalid member access for {:?}", member.get_type())
                }
            }
            _ => panic!("Invalid STORE for {self:?}")
        }
    }

    pub fn load_field(&self, fields: Vec<MemberInfo>, children: HashMap<String, MemberInfo>, generator: &mut BytecodeGen) {
        match self {
            TypedExpr::IdentifierExpr(t, name) => {
                for f in fields.clone() {
                    if f.1 == *name {
                        return match t {
                            AstType::Bool |
                            AstType::Int => generator.i_load_offset(f.2),
                            AstType::Float => generator.f_load_offset(f.2),
                            AstType::StructType { .. } => generator.a_load_offset(f.2),

                            _ => panic!("Cannot LOAD_FIELD for {self:?}")
                        };
                    }
                }

                let value = children.get(name);

                if let Some(member) = value {
                    generator.call(&member.1);
                } else {
                    panic!("Member not found {name}")
                }
            }
            TypedExpr::CallExpr { func, args, .. } => {
                let mut size = 0;
                for a in args {
                    a.generate_bytecode(generator, Load);
                    size += a.get_type().word_size();
                }

                let t = func.get_type();
                match t {
                    AstType::FunctionType {name, .. } => {
                        generator.call(&name);
                    }
                    AstType::StructType {..} => {
                        generator.create_stack_ptr(size);
                    }
                    _ => panic!("Cannot call {t:?}")
                }
            }

            _ => panic!("Cannot call LOAD_FIELD on {self:?}")
        }
    }

    pub fn store_field(&self, fields: Vec<MemberInfo>, generator: &mut BytecodeGen) {
        match self {
            TypedExpr::IdentifierExpr(t, name) => {
                for f in fields.clone() {
                    if f.1 == *name {
                        return match t {
                            AstType::Bool |
                            AstType::Int => generator.i_store_offset(f.2),
                            AstType::Float => generator.f_store_offset(f.2),
                            AstType::StructType { .. } => generator.a_store_offset(f.2),

                            _ => panic!("Cannot STORE_FIELD for {self:?}")
                        };
                    }
                }

                panic!("Invalid STORE_FIELD {name} for {self:?}");
            }

            _ => panic!("Cannot call STORE_FIELD on {self:?}")
        }
    }

}