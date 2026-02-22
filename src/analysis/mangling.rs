use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{FunctionType, FunctionsType};
use crate::ast::expression::Expression::{AssignExpr, BinaryExpr, CallExpr, DecrementExpr, IdentifierExpr, IncrementExpr, MemberExpr, PrefixExpr, TupleExpr};
use crate::ast::expression::TypedExpr;
use crate::ast::statement::Statement::{BlockStmt, ExpressionStmt, FunctionStmt, IfStmt, ReturnStmt, StructStmt, VarDeclarationStmt, WhileStmt};
use crate::ast::statement::{Parameter, Statement, TypedStmt};

impl TypedStmt {


    pub fn mangle_functions(self) -> TypedStmt {
        self._mangle_functions(String::new())
    }

    fn _mangle_functions(self, owner: String) -> TypedStmt {
        match self {
            BlockStmt { body } => {
                let mut mangled_body = vec![];

                for b in body {
                    mangled_body.push(b._mangle_functions(owner.clone()));
                }

                BlockStmt{body: mangled_body}
            }
            ExpressionStmt(e) => ExpressionStmt(e.mangle_functions(owner.clone())),
            VarDeclarationStmt { name, is_const, value, explicit_type } => {
                let mangled_value = value.mangle_functions(owner.clone());

                VarDeclarationStmt {name, is_const, value: mangled_value, explicit_type}
            }
            IfStmt {condition, body, else_branch } => {
                let mangled_condition = condition.mangle_functions(owner.clone());
                let mut mangled_body = vec![];

                for b in body {
                    mangled_body.push(b._mangle_functions(owner.clone()));
                }

                let mangled_branch = match else_branch {
                    None => None,
                    Some(v) => Some(v._mangle_functions(owner.clone()).boxed())
                };

                IfStmt {condition: mangled_condition, body: mangled_body, else_branch: mangled_branch}
            }
            WhileStmt {condition, body } => {
                let mangled_condition = condition.mangle_functions(owner.clone());
                let mut mangled_body = vec![];

                for b in body {
                    mangled_body.push(b._mangle_functions(owner.clone()));
                }

                WhileStmt {condition: mangled_condition, body: mangled_body}
            }
            FunctionStmt { name, params, body, return_type } => {
                let name = from_owner(name, owner.clone());
                let name = mangle_name(name.clone(), &params);

                let mut mangled_body = vec![];

                for b in body {
                    mangled_body.push(b._mangle_functions(owner.clone()));
                }

                FunctionStmt {name, params, body: mangled_body, return_type}
            }
            StructStmt { name, fields, body } => {
                let mut mangled_body = vec![];



                let owner = if owner.is_empty() {
                    name.clone()
                } else {
                    owner + "_" + name.as_str()
                };

                for b in body {
                    mangled_body.push(b._mangle_functions(owner.clone()));
                }

                StructStmt {name, fields, body: mangled_body}
            }
            ReturnStmt(e) => ReturnStmt(e.mangle_functions(owner.clone())),
            TypedStmt::CommentStmt(_) => self,
            TypedStmt::MultilineCommentStmt(_) => self
        }
    }

}


impl TypedExpr {
    fn mangle_functions(self, owner: String) -> TypedExpr {
        match self {
            TypedExpr::BoolLiteralExpr(_) |
            TypedExpr::IntLiteralExpr(_) |
            TypedExpr::FloatLiteralExpr(_) |
            TypedExpr::StringLiteralExpr(_) => self,

            IncrementExpr(t, e) => {
                let t = t.mangle_function(owner.clone());
                IncrementExpr(t, e.mangle_functions(owner.clone()).boxed())
            }
            DecrementExpr(t, e) => {
                let t = t.mangle_function(owner.clone());
                DecrementExpr(t, e.mangle_functions(owner.clone()).boxed())
            }
            PrefixExpr { t, operator, expr } => {
                let t = t.mangle_function(owner.clone());
                let mangled_expr = expr.mangle_functions(owner.clone()).boxed();

                PrefixExpr {t, operator, expr: mangled_expr}
            }
            BinaryExpr { t, left, operator, right } => {
                let t = t.mangle_function(owner.clone());
                let left = left.mangle_functions(owner.clone()).boxed();
                let right = right.mangle_functions(owner.clone()).boxed();

                BinaryExpr {t, left, operator, right}
            }
            TupleExpr { t, values } => {
                let t = t.mangle_function(owner.clone());
                let mut mangled_values = vec![];

                for v in values {
                    mangled_values.push(v.mangle_functions(owner.clone()));
                }

                TupleExpr {t, values: mangled_values}
            }

            AssignExpr { t, assignee, value } => {
                let t = t.mangle_function(owner.clone());
                let assignee = assignee.mangle_functions(owner.clone()).boxed();
                let value = value.mangle_functions(owner.clone()).boxed();

                AssignExpr {t, assignee, value}
            }
            MemberExpr { t, member, property } => {
                let t = t.mangle_function(owner.clone());
                let member = member.mangle_functions(owner.clone()).boxed();
                let property = property.mangle_functions(owner.clone()).boxed();

                MemberExpr {t, member, property}
            }
            CallExpr { t, func, args } => {
                let t = t.mangle_function(owner.clone());
                let func = func.mangle_functions(owner.clone()).boxed();

                let mut mangled_args = vec![];

                for a in args {
                    mangled_args.push(a.mangle_functions(owner.clone()));
                }

                CallExpr {t, func, args: mangled_args}
            }
            IdentifierExpr(t, name) => {
                let t = t.mangle_function(owner.clone());
                if let FunctionType{params, ..} = t.clone() {
                    let name = from_owner(name, owner);
                    let name = mangle_name_type(name.clone(), &params);

                    IdentifierExpr(t, name)
                } else {
                    IdentifierExpr(t, name)
                }
            }
        }
    }
}

impl AstType {

    fn mangle_function(self, owner: String) -> AstType {
        if let FunctionType {name, params, return_type} = self {
            let name = from_owner(name, owner);
            let name = mangle_name_type(name, &params);

            FunctionType {name, params, return_type}
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

fn mangle_name(name: String, params: &Vec<Parameter>) -> String {
    let mut name = name + "_";

    for p in params {
        name += p.param_type.get_type_name().as_str();
    }

    name
}

fn mangle_name_type(name: String, params: &Vec<AstType>) -> String {
    let mut name = name + "_";

    for p in params {
        name += p.get_type_name().as_str();
    }

    name
}