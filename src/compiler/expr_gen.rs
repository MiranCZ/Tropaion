use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType::NullableType;
use crate::ast::ast_type::{AstType, MemberInfo};
use crate::ast::expression::Expression::NullableExpr;
use crate::ast::expression::{deref, Expression, TypedExpr};
use crate::compiler::codegen::BytecodeGen;
use crate::compiler::expr_gen::Operation::{Load, LoadDeref, LoadField, Store, StoreField};
use crate::error::compilation_error::CompilationError::{IllegalBinOperator, InvalidOperator, MemberNotFound, StructTooLarge};
use crate::error::compilation_error::{CompilationError, EmptyRes};
use crate::error::ok;
use crate::error::runtime_error::ValueTypeVariant;
use crate::error::runtime_error::ValueTypeVariant::Nullable;
use crate::lexer::token::SimpleToken;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Operation {
    Load, // loads the expression on top of the stack
    LoadDeref,
    Store, // stores value on top of the stack into the expression
    LoadField {
        fields: Vec<MemberInfo>,
        // fields and methods
        children: HashMap<String, MemberInfo>,
    },
    StoreField {
        fields: Vec<MemberInfo>
    },
    LoadRefOffset(TypedExpr),
    StoreRefOffset(TypedExpr),
}
impl TypedExpr {


    pub fn generate_bytecode(&self,registry: &TypeRegistry ,generator: &mut BytecodeGen, operation: Operation) -> EmptyRes {
        generator.push_span(self.span);
        match operation {
            Load => self.load(registry, generator, false)?,
            LoadDeref => self.load(registry, generator, true)?,
            Store => self.store(registry, generator)?,
            LoadField {fields, children} => {
                self.load_field(registry, fields, children, generator)?;
            }
            StoreField {fields} => {
                self.store_field(registry, fields, generator)?;
            }

            Operation::LoadRefOffset(index) => {
                self.load_ref_offset(registry, index, generator)?;
            }

            Operation::StoreRefOffset(index) => {
                self.store_ref_offset(registry, index, generator)?;
            }

        };
        generator.pop_span();

        ok()
    }

    pub fn load(&self,registry: &TypeRegistry ,generator: &mut BytecodeGen, dereference: bool) -> EmptyRes {
        match &self.node {
            Expression::NullLiteralExpr(_) => {
                generator.null_const();
            }
            Expression::BoolLiteralExpr(_, b) => {
                if *b {
                    generator.i_const(1);
                } else {
                    generator.i_const(0);
                }
            }
            Expression::IntLiteralExpr(_, i) => generator.i_const(*i),
            Expression::FloatLiteralExpr(_, f) => generator.f_const(*f),
            Expression::StringLiteralExpr(..) => {
                todo!()
            }
            Expression::ArrayLiteralExpr(t, values) => {
                generator.heap_alloc(values.len() as u32);

                let mut offset = 0;
                for v in values {
                    generator.dup();
                    v.generate_bytecode(registry, generator, Load)?;

                    generator.swap();

                    generator.store_offset_value(registry, offset, v.get_type())?;

                    offset += 1;
                }
            }
            Expression::IdentifierExpr(t, name) => {
                match t.get(registry) {
                    AstType::Bool |
                    AstType::Int => generator.i_load(name.clone()),
                    AstType::Float => generator.f_load(name.clone()),
                    AstType::StructType { .. } |
                    AstType::ArrayType { .. } |
                    AstType::TupleType { .. } => generator.a_load(name.clone()),
                    AstType::NullableType {underlying: t} => {
                        generator.a_load(name.clone());

                        if dereference {
                            generator.load_offset_value(registry, 0, t)?;
                        }

                    },

                    _ => return Err(CompilationError::unsupported_type(t.get(registry), registry))
                }
            }
            NullableExpr(t, child) => {
                child.generate_bytecode(registry, generator, Load)?;

                let typ;
                if let NullableType{underlying} = t.get(registry) {
                    typ = underlying;
                } else {
                    return Err(CompilationError::type_mismatch(Nullable, t.get(registry), registry));
                }

                generator.store_internal_value(registry, typ)?;

                // FIXME should here be 1 or size of `t`?
                generator.create_stack_ptr(1);
            }
            Expression::IncrementExpr(_, e) => {
                e.generate_bytecode(registry, generator, LoadDeref)?;

                generator.i_const(1);
                generator.add();

                e.generate_bytecode(registry, generator, Store)?;
            }
            Expression::DecrementExpr(_, e) => {
                e.generate_bytecode(registry, generator, LoadDeref)?;

                generator.i_const(1);
                generator.sub();

                e.generate_bytecode(registry, generator, Store)?;
            }
            Expression::PrefixExpr { operator, expr, .. } => {
                match operator {
                    SimpleToken::Dash => {
                        generator.i_const(0);
                        expr.generate_bytecode(registry, generator, LoadDeref)?;
                        generator.sub();
                    },
                    SimpleToken::Exclamation => {
                        expr.generate_bytecode(registry, generator, LoadDeref)?;
                        generator.bool_not()
                    },
                    SimpleToken::Tilde => todo!(),

                    _ => return Err(InvalidOperator(*operator))
                }
            }
            Expression::BinaryExpr {left, operator, right, .. } => {
                match operator {
                    SimpleToken::BoolOr => {
                        left.generate_bytecode(registry, generator, LoadDeref)?;
                        generator.dup();
                        generator.new_skippable_scope_ne();
                        right.generate_bytecode(registry, generator, LoadDeref)?;

                        generator.or();
                        generator.end_scope()?;

                        return ok();
                    }
                    SimpleToken::BoolAnd => {
                        left.generate_bytecode(registry, generator, LoadDeref)?;
                        generator.dup();
                        generator.new_skippable_scope_eq();
                        right.generate_bytecode(registry, generator, LoadDeref)?;

                        generator.and();
                        generator.end_scope()?;

                        return ok();
                    }

                    _ => {}
                }

                if matches!(operator, SimpleToken::Equals) || matches!(operator, SimpleToken::NotEquals) {
                    left.generate_bytecode(registry, generator, Load)?;
                    right.generate_bytecode(registry, generator, Load)?;
                } else {
                    left.generate_bytecode(registry, generator, LoadDeref)?;
                    right.generate_bytecode(registry, generator, LoadDeref)?;
                }

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

                    _ => return Err(IllegalBinOperator(*operator))
                }
            }
            Expression::AssignExpr {assignee, value, .. } => {
                value.generate_bytecode(registry, generator, Load)?;
                assignee.generate_bytecode(registry, generator, Store)?;
            }
            Expression::TupleExpr { values, .. } => {
                for x in values {
                    x.generate_bytecode(registry, generator, Load)?;
                }
            }
            Expression::MemberExpr { member, property, .. } => {
                if let AstType::StructType {fields, children, ..} = deref(member.get_type(), registry) {
                    member.generate_bytecode(registry, generator, LoadDeref)?;

                    property.generate_bytecode(registry, generator, LoadField {
                         fields, children
                    })?;
                } else {
                    return Err(CompilationError::illegal_member_access(member.get_type().get(registry), registry))
                }
            }
            Expression::ArrayAccessExpr {property, index, ..} => {
                property.generate_bytecode(registry, generator, Operation::LoadRefOffset(*index.clone()))?;
            }
            Expression::CallExpr {func, args, .. } => {
                Self::generate_call_expr_load(registry, generator, func, args)?;
            },

            _ => panic!("Invalid LOAD for {self:?}")
        };

        ok()
    }

    fn generate_call_expr_load(registry: &TypeRegistry, generator: &mut BytecodeGen ,func: &Box<TypedExpr>, args: &Vec<TypedExpr>) -> EmptyRes {
        let t = func.get_type();

        fn call(registry: &TypeRegistry, generator: &mut BytecodeGen, args: &Vec<TypedExpr>,typ: TypeEntry) -> EmptyRes {
            match typ.get(registry) {
                AstType::FunctionType { name, .. } => {
                    generator.comment(format!("Loading func {name}, with args {args:?}"));
                    for a in args {
                        a.generate_bytecode(registry, generator, Load)?;
                    }
                    generator.call(&name);
                }

                AstType::StructType {.. } => {
                    let mut size = 0;
                    for a in args {
                        a.generate_bytecode(registry, generator, Load)?;

                        generator.store_internal_value(registry, a.get_type())?;
                        size += a.get_type().get(registry).word_size(registry);
                    }
                    if size > (u16::MAX as u32) {
                        return Err(StructTooLarge(size));
                    }

                    generator.create_stack_ptr(size as u16);
                },
                AstType::NullableType { underlying } => {
                    call(registry, generator, args, underlying)?;
                }

                _ => return Err(CompilationError::illegal_call(typ.get(registry), registry))
            }

            ok()
        }

        call(registry, generator, args, t)?;

        ok()
    }

    pub fn store(&self,registry: &TypeRegistry ,generator: &mut BytecodeGen) -> EmptyRes {
        match &self.node {
            Expression::IdentifierExpr(t, name) => {
                generator.store_value(registry, name, *t)?;
            },
            Expression::IncrementExpr(_, e) => {
                todo!()
            }
            Expression::DecrementExpr(_, e) => {
                todo!()
            }
            Expression::PrefixExpr { .. } => {
                todo!()
            }
            Expression::TupleExpr { values, .. } => {
                panic!("Tuples are immutable!");
            }
            Expression::MemberExpr { member, property, .. } => {
                if let AstType::StructType {fields, ..} = member.get_type().get(registry) {
                    member.generate_bytecode(registry, generator, Load)?;

                    property.generate_bytecode(registry, generator, StoreField {
                        fields
                    })?;
                } else {
                    return Err(CompilationError::illegal_member_access(member.get_type().get(registry), registry))
                }
            }
            Expression::ArrayAccessExpr {property, index, ..} => {
                property.generate_bytecode(registry, generator, Operation::StoreRefOffset(*index.clone()))?;
            }
            _ => panic!("Invalid STORE for {self:?}")
        };

        ok()
    }

    pub fn load_field(&self,registry: &TypeRegistry ,fields: Vec<MemberInfo>, children: HashMap<String, MemberInfo>, generator: &mut BytecodeGen) -> EmptyRes {
        match &self.node {
            Expression::IdentifierExpr(t, name) => {
                for f in fields.clone() {
                    if f.1 == *name {
                        generator.load_offset_value(registry, f.2 as u32, *t)?;

                        return ok();
                    }
                }

                let value = children.get(name);

                if let Some(member) = value {
                    generator.call(&member.1);
                } else {
                    return Err(MemberNotFound(name.clone()));
                }
            }
            Expression::CallExpr { func, args, .. } => {
                Self::generate_call_expr_load(registry, generator, func, args)?;
            }

            _ => panic!("Cannot call LOAD_FIELD on {self:?}")
        };

        ok()
    }

    pub fn store_field(&self,registry: &TypeRegistry ,fields: Vec<MemberInfo>, generator: &mut BytecodeGen) -> EmptyRes {
        match &self.node {
            Expression::IdentifierExpr(t, name) => {
                for f in fields.clone() {
                    if f.1 == *name {
                        generator.store_offset_value(registry, f.2 as u32, *t)?;

                        return ok();
                    }
                }

                panic!("Invalid STORE_FIELD {name} for {self:?}");
            }

            _ => panic!("Cannot call STORE_FIELD on {self:?}")
        }
    }


    pub fn load_ref_offset(&self,registry: &TypeRegistry ,index: TypedExpr, generator: &mut BytecodeGen) -> EmptyRes {
        match &self.node {
            Expression::IdentifierExpr(t, name) => {
                generator.a_load(name.clone());
                index.generate_bytecode(registry, generator, Load)?;

                let typ;

                if let AstType::ArrayType {underlying, ..} = t.get(registry) {
                    typ = underlying;
                } else {
                    return Err(CompilationError::illegal_indexing(t.get(registry), registry));
                }

                generator.load_var_offset(registry, typ)?;
            }

            _ => panic!("Cannot call LOAD_REF on {self:?}")
        }

        ok()
    }


    pub fn store_ref_offset(&self,registry: &TypeRegistry ,index: TypedExpr, generator: &mut BytecodeGen) -> EmptyRes {
        match &self.node {
            Expression::IdentifierExpr(t, name) => {
                generator.a_load(name.clone());
                index.generate_bytecode(registry, generator, Load)?;

                let typ;

                if let AstType::ArrayType {underlying, ..} = t.get(registry) {
                    typ = underlying;
                } else {
                    return Err(CompilationError::illegal_indexing(t.get(registry), registry));
                }

                generator.store_var_offset(registry, typ)?;
            }

            _ => panic!("Cannot call LOAD_REF on {self:?}")
        }

        ok()
    }

}