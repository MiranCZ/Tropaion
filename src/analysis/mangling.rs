use std::collections::HashMap;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::{AstType, MemberInfo};
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType, StructType};
use crate::ast::expression::Expression::{ArrayAccessExpr, ArrayLiteralExpr, AssignExpr, BinaryExpr, CallExpr, DecrementExpr, IdentifierExpr, IncrementExpr, MemberExpr, PrefixExpr, TupleExpr};
use crate::ast::expression::{Expression, TypedExpr};
use crate::ast::statement::Statement::{BlockStmt, ExpressionStmt, FunctionStmt, IfStmt, ReturnStmt, StructStmt, VarDeclarationStmt, WhileStmt};
use crate::ast::statement::{Parameter, Statement, TypedStmt};
use crate::util::spanned::Spanned;

impl TypedStmt {


    pub fn mangle_functions(self, registry: &mut TypeRegistry) -> TypedStmt {
        self._mangle_functions(registry, String::new())
    }

    fn _mangle_functions(self,registry: &mut TypeRegistry, owner: String) -> TypedStmt {
        let stmt = match self.node {
            BlockStmt { body } => {
                let mut mangled_body = vec![];

                for b in body {
                    mangled_body.push(b._mangle_functions(registry, owner.clone()));
                }

                BlockStmt{body: mangled_body}
            }
            ExpressionStmt(e) => ExpressionStmt(e.mangle_functions(registry, owner.clone())),
            VarDeclarationStmt { name, is_const, value, explicit_type } => {
                let mangled_value = value.mangle_functions(registry, owner.clone());

                VarDeclarationStmt {name, is_const, value: mangled_value, explicit_type}
            }
            IfStmt {condition, body, else_branch } => {
                let mangled_condition = condition.mangle_functions(registry, owner.clone());
                let mut mangled_body = vec![];

                for b in body {
                    mangled_body.push(b._mangle_functions(registry, owner.clone()));
                }

                let mangled_branch = match else_branch {
                    None => None,
                    Some(v) => Some(v._mangle_functions(registry, owner.clone()).boxed())
                };

                IfStmt {condition: mangled_condition, body: mangled_body, else_branch: mangled_branch}
            }
            WhileStmt {condition, body } => {
                let mangled_condition = condition.mangle_functions(registry, owner.clone());
                let mut mangled_body = vec![];

                for b in body {
                    mangled_body.push(b._mangle_functions(registry, owner.clone()));
                }

                WhileStmt {condition: mangled_condition, body: mangled_body}
            }
            FunctionStmt { name, params, body, return_type } => {
                let name = from_owner(name, owner.clone());
                let name = mangle_name(registry, name.clone(), &params);

                let mut mangled_body = vec![];

                for b in body {
                    mangled_body.push(b._mangle_functions(registry, owner.clone()));
                }

                FunctionStmt {name, params, body: mangled_body, return_type}
            }
            StructStmt { name, fields, body } => {
                let mut mangled_body = vec![];

                let owner = if owner.is_empty() {
                    name.clone()
                } else {
                    // owner + "_" + name.as_str()
                    // FIXME nested owners shouldn't be possible?
                    owner.clone()
                };

                for b in body {
                    mangled_body.push(b._mangle_functions(registry, owner.clone()));
                }

                StructStmt {name, fields, body: mangled_body}
            }
            ReturnStmt(e) => ReturnStmt(e.mangle_functions(registry, owner.clone())),
            Statement::CommentStmt(_) => self.node,
            Statement::MultilineCommentStmt(_) => self.node
        };
        
        Spanned::of(stmt, self.span)
    }

}


impl TypedExpr {
    fn mangle_functions(self, registry: &mut TypeRegistry, owner: String) -> TypedExpr {
        let expr = match self.node {
            Expression::NullLiteralExpr(..) |
            Expression::BoolLiteralExpr(..) |
            Expression::IntLiteralExpr(..) |
            Expression::FloatLiteralExpr(..) |
            Expression::NullableExpr(..) |
            Expression::StringLiteralExpr(..) => self.node,
            Expression::ArrayLiteralExpr(t, values) => {
                let mut mangled_values = vec![];

                for v in values {
                    mangled_values.push(v.mangle_functions(registry, owner.clone()));
                }
            
                ArrayLiteralExpr(t,mangled_values)
            }

            IncrementExpr(t, e) => {
                t.mangle_function(registry, owner.clone());
                IncrementExpr(t, e.mangle_functions(registry, owner.clone()).boxed())
            }
            DecrementExpr(t, e) => {
                t.mangle_function(registry, owner.clone());
                DecrementExpr(t, e.mangle_functions(registry, owner.clone()).boxed())
            }
            PrefixExpr { t, operator, expr } => {
                t.mangle_function(registry, owner.clone());
                let mangled_expr = expr.mangle_functions(registry, owner.clone()).boxed();

                PrefixExpr {t, operator, expr: mangled_expr}
            }
            BinaryExpr { t, left, operator, right } => {
                t.mangle_function(registry, owner.clone());
                let left = left.mangle_functions(registry, owner.clone()).boxed();
                let right = right.mangle_functions(registry, owner.clone()).boxed();

                BinaryExpr {t, left, operator, right}
            }
            TupleExpr { t, values } => {
                t.mangle_function(registry, owner.clone());
                let mut mangled_values = vec![];

                for v in values {
                    mangled_values.push(v.mangle_functions(registry, owner.clone()));
                }

                TupleExpr {t, values: mangled_values}
            }

            AssignExpr { t, assignee, value } => {
                t.mangle_function(registry, owner.clone());
                let assignee = assignee.mangle_functions(registry, owner.clone()).boxed();
                let value = value.mangle_functions(registry, owner.clone()).boxed();

                AssignExpr {t, assignee, value}
            }
            ArrayAccessExpr {t,property, index} => {
                let property = property.mangle_functions(registry, owner.clone()).boxed();
                let index = index.mangle_functions(registry, owner).boxed();
                
                ArrayAccessExpr {
                    t, property, index
                }
            }
            MemberExpr { t, member, property } => {
                t.mangle_function(registry, owner.clone());
                let member = member.mangle_functions(registry, owner.clone()).boxed();

                let mut repl = owner.clone();

                // println!("MEMBER {:?}",member.get_type());
                
                if let StructType {name, ..} = member.get_type().get(registry) {
                    repl = if owner.is_empty() {
                        name.clone()
                    } else {
                        // owner + "_" + name.as_str()
                        // FIXME nested owners shouldn't be possible?
                        owner.clone()
                    };
                }
                let owner = repl;

                let property = property.mangle_functions(registry, owner.clone()).boxed();

                MemberExpr {t, member, property}
            }
            CallExpr { t, func, args } => {
                t.mangle_function(registry, owner.clone());
                let func = func.mangle_functions(registry, owner.clone()).boxed();

                let mut mangled_args = vec![];

                for a in args {
                    mangled_args.push(a.mangle_functions(registry, owner.clone()));
                }

                CallExpr {t, func, args: mangled_args}
            }
            IdentifierExpr(t, name) => {
                t.mangle_function(registry, owner.clone());
                if let FunctionType{params, ..} = t.get(registry) {
                    let name = from_owner(name, owner);
                    let name = mangle_name_type(registry, name.clone(), &params);

                    IdentifierExpr(t, name)
                } else {
                    IdentifierExpr(t, name)
                }
            }
        };

        Spanned::of(expr, self.span)
    }
}

impl AstType {

    pub fn mangle_function(self,registry: &mut TypeRegistry , owner: String) -> AstType {
        if let FunctionType {name, params, return_type} = self {
            let name = from_owner(name, owner);
            let name = mangle_name_type(registry, name, &params);

            FunctionType {name, params, return_type}
        } else if let FunctionsType {name, overloads} = self {
            let mut mangled = vec![];

            for t in overloads {
                t.mangle_function(registry, owner.clone());
                mangled.push(t);
            }

            FunctionsType {name, overloads:mangled}
        } else if let StructType{name, fields, children} = self {
            let mut mangled_children = HashMap::new();

            let owner = if owner.is_empty() {
                name.clone()
            } else {
                // owner + "_" + name.as_str()
                // FIXME nested owners shouldn't be possible?
                owner.clone()
            };

            for entry in children.iter() {
                let name = entry.0;
                let info = entry.1.clone();

                info.0.mangle_function(registry, owner.clone());

                mangled_children.insert(name.clone(), MemberInfo(info.0, info.1,info.2));
            }

            StructType {name, fields, children: mangled_children}
        } else {
            self
        }
    }

}

fn from_owner(name: String, owner: String) -> String {
    if owner.is_empty() {
        name
    } else {
        owner + "$" + name.as_str()
    }
}

fn mangle_name(registry: &TypeRegistry, name: String, params: &Vec<Parameter>) -> String {
    let mut name = name + "_";

    for p in params {
        name += p.param_type.get(registry).get_type_name(registry).as_str();
    }

    name
}

fn mangle_name_type(registry: &TypeRegistry, name: String, params: &Vec<TypeEntry>) -> String {
    let mut name = name + "_";

    for p in params {
        name += p.get(registry).get_type_name(registry).as_str();
    }

    name
}