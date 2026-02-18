use std::process::id;
use std::string::String;
use crate::analysis::symbol_table::SymbolTable;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{Bool, Float, Int, StringType, SymbolType, TupleType};
use crate::ast::expression::Expression::{AssignExpr, BinaryExpr, BoolLiteralExpr, CallExpr, DecrementExpr, FloatLiteralExpr, IdentifierExpr, IncrementExpr, IntLiteralExpr, MemberExpr, PrefixExpr, StringLiteralExpr, TupleExpr};
use crate::lexer::token::SimpleToken;

pub type UntypedExpr = Expression<()>;
pub type TypedExpr = Expression<AstType>;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<T> {
    BoolLiteralExpr(bool),
    IntLiteralExpr(i32),
    FloatLiteralExpr(f32),
    StringLiteralExpr(String),
    IdentifierExpr(T, String),
    IncrementExpr(T, Box<Expression<T>>),
    DecrementExpr(T, Box<Expression<T>>),
    PrefixExpr {
        t: T,
        operator: SimpleToken,
        expr: Box<Expression<T>>
    },
    BinaryExpr {
        t: T,
        left: Box<Expression<T>>,
        operator: SimpleToken,
        right: Box<Expression<T>>
    },
    AssignExpr {
        t: T,
        assignee: Box<Expression<T>>,
        operator: SimpleToken,
        value: Box<Expression<T>>
    },
    TupleExpr {
        t: T,
        values: Vec<Expression<T>>
    },
    MemberExpr {
        t: T,
        member: Box<Expression<T>>,
        property: Box<Expression<T>>
    },
    CallExpr {
        t: T,
        func: Box<Expression<T>>,
        args: Vec<Expression<T>>
    }
}

impl <T> Expression<T> {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl UntypedExpr {

    pub fn resolve_type(self, symbol_table: &mut SymbolTable) -> TypedExpr {
        fn try_get_resolve_type(symbol_table: &SymbolTable, symbol: &String) -> AstType {
            symbol_table.get_type(symbol.clone()).unwrap_or(SymbolType(symbol.clone()))
        }

        match self {
            BoolLiteralExpr(b) => BoolLiteralExpr(b),
            IntLiteralExpr(i) => IntLiteralExpr(i),
            FloatLiteralExpr(f) => FloatLiteralExpr(f),
            StringLiteralExpr(s) => StringLiteralExpr(s),
            IdentifierExpr(_, identifier) => {
                let t = try_get_resolve_type(symbol_table, &identifier);

                return IdentifierExpr(t, identifier);
            }
            IncrementExpr(_, expr) => {
                let typed = expr.resolve_type(symbol_table);

                return IncrementExpr(typed.get_type(), typed.boxed());
            }
            DecrementExpr(_, expr) => {
                let typed = expr.resolve_type(symbol_table);

                return DecrementExpr(typed.get_type(), typed.boxed());
            }
            PrefixExpr { operator, expr,.. } => {
                let typed = expr.resolve_type(symbol_table);

                return PrefixExpr {t: typed.get_type(), operator, expr: typed.boxed()};
            }
            BinaryExpr {left, operator, right, .. } => {
                let typed_left = left.resolve_type(symbol_table);
                let typed_right = right.resolve_type(symbol_table);

                // FIXME this is wrong, should instead record operator and type combinations (eq. `int + int` => `int`, `int < int` => `bool`)
                if typed_left.get_type() != typed_right.get_type() {
                    panic!("Binary expr arms do not match {:?} vs {:?}",typed_right, typed_left);
                }
                let t = typed_left.get_type();

                return BinaryExpr {t, left: typed_left.boxed(), operator, right:typed_right.boxed()};
            }
            AssignExpr {assignee, operator, value, .. } => {
                let typed_assignee = assignee.resolve_type(symbol_table);
                let typed_value = value.resolve_type(symbol_table);

                if typed_assignee.get_type() != typed_value.get_type() {
                    panic!("Assignment expr arms do not match {:?} vs {:?}",typed_assignee, typed_value);
                }
                let t = typed_assignee.get_type();

                return AssignExpr {t, assignee: typed_assignee.boxed(), operator, value: typed_value.boxed()};
            }
            TupleExpr {values, .. } => {
                let mut types = vec![];

                for v in values {
                    types.push(v.resolve_type(symbol_table));
                }


                let t = TupleType(types.iter().map(|e| e.get_type()).collect());

                return TupleExpr {t, values: types};
            }
            MemberExpr {member, property , .. } => {
                let member_type = member.resolve_type(symbol_table);

                // if we are accessing something on a struct, temporarily add the structs methods and fields into scope
                let mut struct_scope = false;
                if let AstType::StructType{children, ..} = member_type.get_type() {
                    struct_scope = true;
                    symbol_table.push();
                    for x in children {
                        symbol_table.record_type(x.0, x.1);
                    }
                }

                let property_type = property.resolve_type(symbol_table);

                // drop the struct scope
                if struct_scope {
                    symbol_table.pop();
                }

                return MemberExpr {
                    t: property_type.get_type(),
                    member: member_type.boxed(),
                    property: property_type.boxed()
                };
            }
            CallExpr {func, args, .. } => {
                let resolved_func = func.resolve_type(symbol_table);

                if let AstType::FunctionType {return_type, ..} = resolved_func.get_type() {
                    let mut resolved_args = vec![];

                    for arg in args {
                        resolved_args.push(arg.resolve_type(symbol_table));
                    }


                    return CallExpr {
                        t: *return_type,
                        func: resolved_func.boxed(),
                        args: resolved_args
                    };
                }

                panic!("Calling something else than a function! {:?}", resolved_func);
            }
        }
    }

}

impl TypedExpr {
    pub fn get_type(&self) -> AstType {
        match self {
            BoolLiteralExpr(_) => Bool,
            IntLiteralExpr(_) => Int,
            FloatLiteralExpr(_) => Float,
            StringLiteralExpr(_) => StringType,
            IdentifierExpr(t, _) => t.clone(),
            IncrementExpr(t, _) => t.clone(),
            DecrementExpr(t, _) => t.clone(),
            PrefixExpr {t, .. } => t.clone(),
            BinaryExpr {t, .. } => t.clone(),
            AssignExpr {t, .. } => t.clone(),
            TupleExpr {t, .. } => t.clone(),
            MemberExpr {t, .. } => t.clone(),
            CallExpr {t, .. } => t.clone()
        }
    }
}

pub fn bool(b: bool) -> UntypedExpr {
    BoolLiteralExpr(b)
}

pub fn int(i: i32) -> UntypedExpr {
    IntLiteralExpr(i)
}

pub fn float(f: f32) -> UntypedExpr {
    FloatLiteralExpr(f)
}

pub fn string(s: String) -> UntypedExpr {
    StringLiteralExpr(s)
}

pub fn identifier(identifier: String) -> UntypedExpr {
    IdentifierExpr((), identifier)
}

pub fn increment(expr: UntypedExpr) -> UntypedExpr {
    IncrementExpr((), expr.boxed())
}

pub fn decrement(expr: UntypedExpr) -> UntypedExpr {
    DecrementExpr((), expr.boxed())
}

pub fn prefix(operator: SimpleToken, expr: UntypedExpr) -> UntypedExpr {
    PrefixExpr {
        t: (),
        operator,
        expr: expr.boxed()
    }
}

pub fn binary(left: UntypedExpr, operator: SimpleToken, right: UntypedExpr) -> UntypedExpr {
    BinaryExpr {
        t: (),
        left: left.boxed(),
        operator,
        right: right.boxed()
    }
}

pub fn assign(assignee: UntypedExpr, operator: SimpleToken, value: UntypedExpr) -> UntypedExpr {
    AssignExpr {
        t: (),
        assignee: assignee.boxed(),
        operator,
        value: value.boxed()
    }
}

pub fn tuple(values: Vec<UntypedExpr>) -> UntypedExpr {
    TupleExpr {
        t: (),
        values
    }
}

pub fn member(member: UntypedExpr, property: UntypedExpr) -> UntypedExpr {
    MemberExpr {
        t: (),
        member: member.boxed(),
        property: property.boxed()
    }
}

pub fn call(func: UntypedExpr, args: Vec<UntypedExpr>) -> UntypedExpr {
    CallExpr {
        t: (),
        func: func.boxed(),
        args
    }
}
