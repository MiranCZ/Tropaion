use crate::analysis::symbol_table::TypeSymTable;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{ArrayType, Bool, Float, FunctionsType, Int, NullableType, StringType, TupleType, UnknownType, Void};
use crate::ast::expression::Expression::{ArrayAccessExpr, ArrayLiteralExpr, AssignExpr, BinaryExpr, BoolLiteralExpr, CallExpr, DecrementExpr, FloatLiteralExpr, IdentifierExpr, IncrementExpr, IntLiteralExpr, MemberExpr, NullLiteralExpr, NullableExpr, PrefixExpr, StringLiteralExpr, TupleExpr};
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::{Ampersand, Assign, BitXor, Dash, LeftLeft, Percent, Plus, RightRight, Slash, Star, VerticalBar};
use std::string::String;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::error::analysis_error::AnalysisError;
use crate::error::context::{ErrorContext, Span};
use crate::lexer::token::Token::Identifier;
use crate::util::spanned::Spanned;

pub type UntypedExpr = Spanned<Expression<()>>;
pub type TypedExpr = Spanned<Expression<TypeEntry>>;

type SpannedExpr<T> = Spanned<Expression<T>>;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<T> {
    NullLiteralExpr(T),
    BoolLiteralExpr(T, bool),
    IntLiteralExpr(T, i32),
    FloatLiteralExpr(T, f32),
    StringLiteralExpr(T, String),
    ArrayLiteralExpr(T, Vec<SpannedExpr<T>>),
    IdentifierExpr(T, String),
    NullableExpr(T, Box<SpannedExpr<T>>),
    IncrementExpr(T, Box<SpannedExpr<T>>),
    DecrementExpr(T, Box<SpannedExpr<T>>),
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
        property: Box<SpannedExpr<T>>
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

impl UntypedExpr {

    pub fn resolve_type(self, registry: &mut TypeRegistry, symbol_table: &mut TypeSymTable) -> Result<TypedExpr, ErrorContext<AnalysisError>> {
        let ctx = |err| {
            ErrorContext::of(err, self.span)
        };

        let expr = match self.node {
            NullableExpr(..) => panic!("internal API"),

            NullLiteralExpr(_) => {
                let unknown = registry.register(UnknownType);
                
                let t= NullableType {underlying: unknown};
                
                let nullable_type = registry.register(t);
                
                NullLiteralExpr(nullable_type)
            },
            BoolLiteralExpr(_,b) => {
                let t = Bool;
                let reg = registry.register(t);

                BoolLiteralExpr(reg, b)
            },
            IntLiteralExpr(_,i) => {
                let t = Int;
                let reg = registry.register(t);

                IntLiteralExpr(reg, i)
            },
            FloatLiteralExpr(_,f) => {
                let t = Float;
                let reg = registry.register(t);

                FloatLiteralExpr(reg, f)
            }
            StringLiteralExpr(_, s) => {
                let t = StringType;
                let reg = registry.register(t);

                StringLiteralExpr(reg, s)
            },
            ArrayLiteralExpr(_, values) => {
                if values.is_empty() {
                    let typ = ArrayType {underlying: registry.register(UnknownType)};

                    ArrayLiteralExpr(registry.register(typ), vec![])
                } else {
                    let mut typed_values = vec![];

                    for v in values {
                        typed_values.push(v.resolve_type(registry, symbol_table)?);
                    }

                    let mut underlying = typed_values[0].get_type();

                    for v in &typed_values {
                        if let Some(res) = underlying.get(registry).get_assign_result(v.get_type().get(registry), registry) {
                            underlying = registry.register(res);
                        } else if let Some(res) = v.get_type().get(registry).get_assign_result(underlying.get(registry), registry) {
                            underlying = registry.register(res);
                        }
                    }

                    for v in typed_values.iter_mut() {
                        v.set_type(registry, underlying.get(registry))
                    }

                    let array_type = ArrayType { underlying };

                    ArrayLiteralExpr(registry.register(array_type), typed_values)
                }
            }
            IdentifierExpr(_, identifier) => {
                let v = symbol_table.get_with_info(identifier.clone());

                if let Some(tuple) = v{
                    let t = tuple.0;
                    let info = tuple.1;

                    if let Some(v) = info && v {
                        MemberExpr {
                            t: t.clone(),
                            member: Spanned::new(IdentifierExpr((), "this".to_string()), self.span.from, self.span.from).resolve_type(registry, symbol_table)?.boxed(),
                            property: Spanned::of(IdentifierExpr(t, identifier), self.span).boxed()
                        }
                    } else {
                        IdentifierExpr(t, identifier)
                    }
                } else {
                    return Err(ctx(AnalysisError::UnknownType(identifier)));
                }
            }
            IncrementExpr(_, expr) => {
                let typed = expr.resolve_type(registry, symbol_table)?;

                IncrementExpr(typed.get_type(), typed.boxed())
            }
            DecrementExpr(_, expr) => {
                let typed = expr.resolve_type(registry, symbol_table)?;

                DecrementExpr(typed.get_type(), typed.boxed())
            }
            PrefixExpr { operator, expr,.. } => {
                let typed = expr.resolve_type(registry, symbol_table)?;

                PrefixExpr {t: typed.get_type(), operator, expr: typed.boxed()}
            }
            BinaryExpr {left, operator, right, .. } => {
                let typed_left = left.resolve_type(registry, symbol_table)?;
                let typed_right = right.resolve_type(registry, symbol_table)?;

                let result_type = symbol_table.op_table.get_op_result(registry, typed_left.get_type(), operator, typed_right.get_type());

                let result_type = if let Ok(t) = result_type {
                    t
                } else {
                    return Err(ctx(result_type.err().unwrap()));
                };

                let t = registry.register(result_type);

                BinaryExpr {t, left: typed_left.boxed(), operator, right:typed_right.boxed()}
            }
            AssignExpr {assignee, value, .. } => {
                let mut typed_assignee = assignee.resolve_type(registry, symbol_table)?;
                let mut typed_value = value.resolve_type(registry, symbol_table)?;

                let assign_result = typed_assignee.get_type().get(registry).get_assign_result(typed_value.get_type().get(registry), registry);
                if let Some(t) = assign_result{
                    typed_assignee.set_type(registry, t.clone());
                    typed_value.set_type(registry, t);
                } else {
                    return Err(ctx(AnalysisError::illegal_type_assignment(typed_assignee.get_type(), typed_value.get_type(), registry)));
                }

                let t = typed_assignee.get_type();

                AssignExpr {t, assignee: typed_assignee.boxed(), value: typed_value.boxed()}
            }
            Expression::ArrayAccessExpr {property, index, ..} => {
                let property = property.resolve_type(registry, symbol_table)?;
                let index = index.resolve_type(registry, symbol_table)?;

                let underlying;
                if let ArrayType {underlying: u} = property.get_type().get(registry) {
                    underlying = u;
                } else {
                    return Err(ctx(AnalysisError::illegal_indexing(property.get_type(), registry)));
                }

                ArrayAccessExpr {
                    t: underlying,
                    property: property.boxed(),
                    index: index.boxed()
                }
            }
            TupleExpr {values, .. } => {
                let mut types = vec![];

                for v in values {
                    types.push(v.resolve_type(registry, symbol_table)?);
                }


                let t = TupleType(types.iter().map(|e| e.get_type()).collect());

                let tuple_type = registry.register(t);

                TupleExpr {t: tuple_type, values: types}
            }
            MemberExpr {member, property , .. } => {
                let member_type = member.resolve_type(registry, symbol_table)?;

                // if we are accessing something on a struct, temporarily add the structs methods and fields into scope
                let mut struct_scope = false;
                if let AstType::StructType{children, ..} = deref(member_type.get_type(), registry) {
                    struct_scope = true;
                    symbol_table.push();
                    for x in children {
                        symbol_table.record(x.0, x.1.0);
                    }
                }

                let property_type = property.resolve_type(registry, symbol_table)?;

                // drop the struct scope
                if struct_scope {
                    symbol_table.pop();
                }

                MemberExpr {
                    t: property_type.get_type(),
                    member: member_type.boxed(),
                    property: property_type.boxed()
                }
            }
            CallExpr {func, args, .. } => {
                Self::resolve_call_expr(self.span, registry, symbol_table, func, args)?
            }
        };

        Ok(Spanned::of(expr, self.span))
    }

    fn resolve_call_expr(span: Span, registry: &mut TypeRegistry, symbol_table: &mut TypeSymTable, func: Box<SpannedExpr<()>>, args: Vec<SpannedExpr<()>>) -> Result<Expression<TypeEntry>, ErrorContext<AnalysisError>> {
        let mut resolved_func = func.clone().resolve_type(registry, symbol_table)?;

        if let FunctionsType { overloads, name, .. } = resolved_func.get_type().get(registry) {
            let mut resolved_args = vec![];

            for arg in args.clone() {
                resolved_args.push(arg.resolve_type(registry, symbol_table)?);
            }

            // FIXME leaking the void here a bit
            let mut func = registry.register(Void);

            'overloadLoop:
            for overload in overloads.iter() {
                if let AstType::FunctionType { name, params, .. } = overload.get(registry) {
                    if params.len() != resolved_args.len() {
                        continue;
                    }

                    for i in 0..resolved_args.len() {
                        if !params[i].get(registry).loose_equals(&resolved_args[i].get_type().get(registry), registry) {
                            continue 'overloadLoop;
                        }
                    }

                    func = *overload;
                    break;
                } else {
                    panic!();
                }
            }


            if let AstType::FunctionType { return_type, params, .. } = func.get(registry) {
                // resolve argument types
                for i in 0..params.len() {
                    let arg = &mut resolved_args[i];
                    let p = params[i];

                    if let Some(r) = p.get(registry).get_assign_result(arg.get_type().get(registry), registry) {
                        arg.set_type(registry, r);
                    } else {
                        return Err(ErrorContext::of(AnalysisError::illegal_type_assignment(p, arg.get_type(), registry),arg.span));
                    }
                }

                // FIXME not at all sure if `set_type` or `change_type` should be called here aaaa
                resolved_func.change_type(registry, func.get(registry));

                if let MemberExpr { t, member, property } = &resolved_func.node
                    && let IdentifierExpr(t, name) = &member.node && name == "this"
                {
                    let return_type = return_type.duplicate(registry);

                    let mut property = property.clone();
                    property.change_type(registry, func.get(registry));

                    let res = MemberExpr {
                        t: return_type,
                        member: member.clone().boxed(),
                        property: Spanned::of(CallExpr {
                            t: return_type,
                            func: property.clone(),
                            args: resolved_args

                            // FIXME I think the span should be combined with args here
                        }, property.span).boxed()
                    };

                    return Ok(res);
                }

                return Ok(CallExpr {
                    t: return_type.duplicate(registry),
                    func: resolved_func.boxed(),
                    args: resolved_args
                });
            }
        }

        // calling constructor of a struct
        if let AstType::StructType { fields, .. } = resolved_func.get_type().get(registry) {
            let mut resolved_args = vec![];

            for arg in args {
                resolved_args.push(arg.resolve_type(registry, symbol_table)?);
            }

            if fields.len() != resolved_args.len() {
                panic!("Invalid constructor call");
            }

            for i in 0..fields.len() {
                let f = &fields[i].0;
                let a = &mut resolved_args[i];

                if let Some(ass_res) = f.get(registry).get_assign_result(a.get_type().get(registry), registry) {
                    a.set_type(registry, ass_res);
                }
            }

            return Ok(CallExpr {
                t: resolved_func.get_type().duplicate(registry),
                func: resolved_func.boxed(),
                args: resolved_args
            });
        }

        Err(ErrorContext::of(AnalysisError::illegal_call(resolved_func.get_type(), registry), span))
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
            NullLiteralExpr(t) => *t,
            BoolLiteralExpr(t, ..) => *t,
            IntLiteralExpr(t, ..) => *t,
            FloatLiteralExpr(t, ..) => *t,
            StringLiteralExpr(t, ..) => *t,
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

pub fn int(i: i32) -> Expr {
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

pub fn member(member: UntypedExpr, property: UntypedExpr) -> Expr {
    MemberExpr {
        t: (),
        member: member.boxed(),
        property: property.boxed()
    }
}

pub fn call(func: UntypedExpr, args: Vec<UntypedExpr>) -> Expr {
    CallExpr {
        t: (),
        func: func.boxed(),
        args
    }
}
