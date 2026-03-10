use crate::analysis::symbol_table::TypeSymTable;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{ArrayType, Bool, Float, FunctionsType, Int, NullableType, StringType, TupleType, UnknownType, Void};
use crate::ast::expression::Expression::{ArrayAccessExpr, ArrayLiteralExpr, AssignExpr, BinaryExpr, BoolLiteralExpr, CallExpr, DecrementExpr, ErroredExpr, FloatLiteralExpr, IdentifierExpr, IncrementExpr, IntLiteralExpr, MemberExpr, NullDerefExpr, NullLiteralExpr, NullableExpr, PrefixExpr, StringLiteralExpr, TupleExpr};
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::{Ampersand, Assign, BitXor, Dash, LeftLeft, Percent, Plus, RightRight, Slash, Star, VerticalBar};
use std::string::String;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::error::analysis_error::AnalysisError;
use crate::error::analysis_error::AnalysisError::IllegalNullDeref;
use crate::error::context::{ErrorContext, Span};
use crate::lexer::token::Token::Identifier;
use crate::util::spanned::Spanned;

pub type UntypedExpr = Spanned<Expression<()>>;
pub type TypedExpr = Spanned<Expression<TypeEntry>>;

type SpannedExpr<T> = Spanned<Expression<T>>;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<T> {
    ErroredExpr(T),
    
    NullLiteralExpr(T),
    BoolLiteralExpr(T, bool),
    IntLiteralExpr(T, i64),
    FloatLiteralExpr(T, f32),
    StringLiteralExpr(T, String),
    ArrayLiteralExpr(T, Vec<SpannedExpr<T>>),
    IdentifierExpr(T, String),
    NullableExpr(T, Box<SpannedExpr<T>>),
    IncrementExpr(T, Box<SpannedExpr<T>>),
    DecrementExpr(T, Box<SpannedExpr<T>>),
    NullDerefExpr(T, Box<SpannedExpr<T>>),
    PrefixExpr {
        t: T,
        operator: SimpleToken,
        expr: Box<SpannedExpr<T>>
    },
    BinaryExpr {
        t: T,
        left: Box<SpannedExpr<T>>,
        operator: SimpleToken,
        right: Box<SpannedExpr<T>>
    },
    AssignExpr {
        t: T,
        assignee: Box<SpannedExpr<T>>,
        value: Box<SpannedExpr<T>>
    },
    TupleExpr {
        t: T,
        values: Vec<SpannedExpr<T>>
    },
    ArrayAccessExpr {
        t: T,
        property: Box<SpannedExpr<T>>,
        index: Box<SpannedExpr<T>>
    },
    MemberExpr {
        t: T,
        member: Box<SpannedExpr<T>>,
        property: Box<SpannedExpr<T>>,
        null_safe: bool
    },
    CallExpr {
        t: T,
        func: Box<SpannedExpr<T>>,
        args: Vec<SpannedExpr<T>>
    }
}

impl <T> Expression<T> {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}


pub fn box_arg(registry: &mut TypeRegistry, arg: &mut TypedExpr, desired: TypeEntry) {
    // arg does not know its type
    if matches!(arg.get_type().get(registry), UnknownType) {
        arg.set_type(registry, desired.get(registry));
        return;
    }

    // arg is '(<unknown>)?'
    if let NullableType {underlying} = arg.get_type().get(registry) && matches!(underlying.get(registry), UnknownType) {
        if !matches!(desired.get(registry), NullableType {..}) {
            panic!("AAAA WTF");
        }

        arg.set_type(registry, desired.get(registry));
        return;
    }

    if matches!(desired.get(registry), NullableType {..}) && !matches!(arg.get_type().get(registry), NullableType {..}) {
        *arg = Spanned::of(NullableExpr(registry.register(NullableType { underlying: arg.get_type() }), arg.clone().boxed()), arg.span);
    }
}

pub fn deref(t: TypeEntry, registry: &TypeRegistry) -> AstType {
    match t.get(registry) {
        NullableType { underlying } => {
            deref(underlying, registry)
        }

        _ => t.get(registry)
    }
}

impl TypedExpr {
    pub fn get_type(&self) -> TypeEntry {
        match &self.node {
            ErroredExpr(t) => *t,
            
            NullLiteralExpr(t) => *t,
            BoolLiteralExpr(t, ..) => *t,
            IntLiteralExpr(t, ..) => *t,
            FloatLiteralExpr(t, ..) => *t,
            StringLiteralExpr(t, ..) => *t,
            NullDerefExpr(t, ..) => *t,
            ArrayLiteralExpr(t, ..) => *t,
            IdentifierExpr(t, _) => *t,
            NullableExpr(t,_) => *t,
            IncrementExpr(t, _) => *t,
            DecrementExpr(t, _) => *t,
            PrefixExpr {t, .. } => *t,
            BinaryExpr {t, .. } => *t,
            AssignExpr {t, .. } => *t,
            ArrayAccessExpr {t, .. } => *t,
            TupleExpr {t, .. } => *t,
            MemberExpr {t, .. } => *t,
            CallExpr {t, .. } =>* t
        }
    }
}

impl TypedExpr {
    pub fn set_type(&mut self,registry: &mut TypeRegistry ,typ: AstType) {
        if self.get_type().get(registry).equals(&typ, registry) {
            return;
        }

        // cast type to a nullable one
        if let NullableType {underlying} = typ {

            // already null-like, only set underlying value
            if let NullLiteralExpr(t) = &mut self.node {
                t.mutate(registry, NullableType {underlying});

                return;
            }

            // set underlying type
            self.set_type(registry, underlying.get(registry));

            // make the underlying type mutable
            self.get_type().mutate(registry, NullableType {underlying});

            // make expression mutable
            self.node = NullableExpr(self.get_type(),self.clone().boxed());

            return;
        }


        match &mut self.node {
            ErroredExpr(..) => panic!(),
            
            BoolLiteralExpr(..) => panic!(),
            FloatLiteralExpr(..) => panic!("{typ:?}"),
            StringLiteralExpr(..) => panic!(),
            IntLiteralExpr(t, v) => {
                t.mutate(registry, Float);
                
                self.node = FloatLiteralExpr(*t, *v as f32)
            },
            NullLiteralExpr(t) => {
                if let NullableType {..} = typ {
                    t.mutate(registry, typ);
                } else {
                    panic!("was set to {typ:?}")
                }
            }
            NullDerefExpr(t, ..) |
            ArrayLiteralExpr(t, ..) |
            IdentifierExpr(t, _) |
            NullableExpr(t, _) |
            IncrementExpr(t, _) |
            DecrementExpr(t, _) |
            PrefixExpr {t, .. } |
            BinaryExpr {t, .. } |
            AssignExpr {t, .. } |
            ArrayAccessExpr {t, .. } |
            TupleExpr {t, .. } |
            MemberExpr {t, .. } |
            CallExpr {t, .. } => {
                t.mutate(registry, typ);
            }
        }
    }


    /// does not mutate the original type, but reassigns a new one
    pub fn change_type(&mut self,registry: &mut TypeRegistry ,typ: AstType) {
        let new_type = registry.register(typ.clone());

        match &mut self.node {
            ErroredExpr(..) => panic!(),
            
            BoolLiteralExpr(..) => panic!(),
            FloatLiteralExpr(..) => panic!(),
            StringLiteralExpr(..) => panic!(),
            IntLiteralExpr(t, v) => {
                self.node = FloatLiteralExpr(registry.register(Float),*v as f32)
            },
            NullLiteralExpr(t) => {
                if let NullableType {..} = typ {
                    *t = new_type;
                } else {
                    panic!()
                }
            }
            NullDerefExpr(t, _) |
            ArrayLiteralExpr(t, _) |
            NullableExpr(t, _) |
            IdentifierExpr(t, _) |
            IncrementExpr(t, _) |
            DecrementExpr(t, _) |
            PrefixExpr {t, .. }|
            BinaryExpr {t, .. }|
            AssignExpr {t, .. }|
            ArrayAccessExpr {t, .. }|
            TupleExpr {t, .. }|
            MemberExpr {t, .. }|
            CallExpr {t, .. } => {
                *t = new_type;
            }
        }
    }

}

type Expr = Expression<()>;

pub fn bool(b: bool) -> Expr {
    BoolLiteralExpr((), b)
}

pub fn int(i: i64) -> Expr {
    IntLiteralExpr((), i)
}

pub fn float(f: f32) -> Expr {
    FloatLiteralExpr((), f)
}

pub fn string(s: String) -> Expr {
    StringLiteralExpr((), s)
}

pub fn identifier(identifier: String) -> Expr {
    IdentifierExpr((), identifier)
}

pub fn array_literal(values: Vec<UntypedExpr>) -> Expr {
    ArrayLiteralExpr((), values)
}

pub fn increment(expr: UntypedExpr) -> Expr {
    IncrementExpr((), expr.boxed())
}

pub fn decrement(expr: UntypedExpr) -> Expr {
    DecrementExpr((), expr.boxed())
}

pub fn null_deref(expr: UntypedExpr) -> Expr {
    NullDerefExpr((), expr.boxed())
}

pub fn prefix(operator: SimpleToken, expr: UntypedExpr) -> Expr {
    PrefixExpr {
        t: (),
        operator,
        expr: expr.boxed()
    }
}

pub fn binary(left: UntypedExpr, operator: SimpleToken, right: UntypedExpr) -> Expr {
    BinaryExpr {
        t: (),
        left: left.boxed(),
        operator,
        right: right.boxed()
    }
}

pub fn assign(assignee: UntypedExpr, operator: SimpleToken, value: UntypedExpr) -> Expr {
    let binary = |op| {
        assign(assignee.clone(), Assign, Spanned::of(binary(assignee.clone(),op,value.clone()), Span::combined(assignee.span, value.span)))
    };

    match operator {
        Assign => {
            AssignExpr {
                t: (),
                assignee: assignee.boxed(),
                value: value.boxed()
            }
        }
        SimpleToken::PlusAssign => binary(Plus),
        SimpleToken::DashAssign => binary(Dash),
        SimpleToken::StarAssign => binary(Star),
        SimpleToken::SlashAssign => binary(Slash),
        SimpleToken::PercentAssign => binary(Percent),
        SimpleToken::RightRightAssign => binary(RightRight),
        SimpleToken::LeftLeftAssign => binary(LeftLeft),
        SimpleToken::VertBarAssign => binary(VerticalBar),
        SimpleToken::AmpersandAssign => binary(Ampersand),
        SimpleToken::BitXorAssign => binary(BitXor),
        _ => panic!("Not an implement assignment operator! {operator:?}")
    }
}

pub fn array_access(property: UntypedExpr, index: UntypedExpr) -> Expr {
    ArrayAccessExpr {
        t: (),
        property: property.boxed(),
        index: index.boxed()
    }
}

pub fn tuple(values: Vec<UntypedExpr>) -> Expr {
    TupleExpr {
        t: (),
        values
    }
}

pub fn member(member: UntypedExpr, property: UntypedExpr, null_safe: bool) -> Expr {
    MemberExpr {
        t: (),
        member: member.boxed(),
        property: property.boxed(),
        null_safe
    }
}

pub fn call(func: UntypedExpr, args: Vec<UntypedExpr>) -> Expr {
    CallExpr {
        t: (),
        func: func.boxed(),
        args
    }
}
