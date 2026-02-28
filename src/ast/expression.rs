use crate::analysis::symbol_table::TypeSymTable;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{Bool, Float, FunctionsType, Int, NullableType, StringType, TupleType, UnknownType, Void};
use crate::ast::expression::Expression::{AssignExpr, BinaryExpr, BoolLiteralExpr, CallExpr, DecrementExpr, FloatLiteralExpr, IdentifierExpr, IncrementExpr, IntLiteralExpr, MemberExpr, NullLiteralExpr, NullableExpr, PrefixExpr, StringLiteralExpr, TupleExpr};
use crate::lexer::token::SimpleToken;
use crate::lexer::token::SimpleToken::{Ampersand, Assign, BitXor, Dash, LeftLeft, Percent, Plus, RightRight, Slash, Star, VerticalBar};
use std::string::String;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::lexer::token::Token::Identifier;

pub type UntypedExpr = Expression<()>;
pub type TypedExpr = Expression<TypeEntry>;

#[derive(Debug, PartialEq, Clone)]
pub enum Expression<T> {
    NullLiteralExpr(T),
    BoolLiteralExpr(T, bool),
    IntLiteralExpr(T, i32),
    FloatLiteralExpr(T, f32),
    StringLiteralExpr(T, String),
    IdentifierExpr(T, String),
    NullableExpr(T, Box<Expression<T>>),
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

    pub fn resolve_type(self, registry: &mut TypeRegistry, symbol_table: &mut TypeSymTable) -> TypedExpr {
        match self {
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
            IdentifierExpr(_, identifier) => {
                let v = symbol_table.get_with_info(identifier.clone());

                if let Some(tuple) = v{
                    let t = tuple.0;
                    let info = tuple.1;

                    if let Some(v) = info && v {
                        return MemberExpr {
                            t: t.clone(),
                            member: IdentifierExpr((), "this".to_string()).resolve_type(registry, symbol_table).boxed(),
                            property: IdentifierExpr(t, identifier).boxed()
                        };
                    } else {
                        return IdentifierExpr(t, identifier);
                    }
                }
                panic!("Failed to resolve symbol {identifier}");

            }
            IncrementExpr(_, expr) => {
                let typed = expr.resolve_type(registry, symbol_table);

                return IncrementExpr(typed.get_type(), typed.boxed());
            }
            DecrementExpr(_, expr) => {
                let typed = expr.resolve_type(registry, symbol_table);

                return DecrementExpr(typed.get_type(), typed.boxed());
            }
            PrefixExpr { operator, expr,.. } => {
                let typed = expr.resolve_type(registry, symbol_table);

                return PrefixExpr {t: typed.get_type(), operator, expr: typed.boxed()};
            }
            BinaryExpr {left, operator, right, .. } => {
                let typed_left = left.resolve_type(registry, symbol_table);
                let typed_right = right.resolve_type(registry, symbol_table);

                let result_type = symbol_table.op_table.get_op_result(registry, typed_left.get_type().get(registry), operator, typed_right.get_type().get(registry));

                if result_type.is_none() {
                    panic!("Not a valid binary expression {:?} {:?} {:?} {registry:#?}",typed_left.get_type(), operator, typed_right.get_type());
                }
                let t = registry.register(result_type.unwrap());

                return BinaryExpr {t, left: typed_left.boxed(), operator, right:typed_right.boxed()};
            }
            AssignExpr {assignee, value, .. } => {
                let mut typed_assignee = assignee.resolve_type(registry, symbol_table);
                let mut typed_value = value.resolve_type(registry, symbol_table);

                let assign_result = typed_assignee.get_type().get(registry).get_assign_result(typed_value.get_type().get(registry), registry);
                if let Some(t) = assign_result{
                    typed_assignee.set_type(registry, t.clone());
                    typed_value.set_type(registry, t);
                } else {
                    panic!("Assignment expr arms do not match {:?} vs {:?} \n{:#?}",typed_assignee.get_type(), typed_value.get_type(), registry);
                }

                let t = typed_assignee.get_type();

                return AssignExpr {t, assignee: typed_assignee.boxed(), value: typed_value.boxed()};
            }
            TupleExpr {values, .. } => {
                let mut types = vec![];

                for v in values {
                    types.push(v.resolve_type(registry, symbol_table));
                }


                let t = TupleType(types.iter().map(|e| e.get_type()).collect());

                let tuple_type = registry.register(t);

                return TupleExpr {t: tuple_type, values: types};
            }
            MemberExpr {member, property , .. } => {
                let member_type = member.resolve_type(registry, symbol_table);

                // if we are accessing something on a struct, temporarily add the structs methods and fields into scope
                let mut struct_scope = false;
                if let AstType::StructType{children, ..} = member_type.get_type().get(registry) {
                    struct_scope = true;
                    symbol_table.push();
                    for x in children {
                        symbol_table.record(x.0, x.1.0);
                    }
                }

                let property_type = property.resolve_type(registry, symbol_table);

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
                let mut resolved_func = func.clone().resolve_type(registry, symbol_table);

                if let FunctionsType {overloads, ..} = resolved_func.get_type().get(registry) {
                    let mut resolved_args = vec![];

                    for arg in args.clone() {
                        resolved_args.push(arg.resolve_type(registry, symbol_table));
                    }

                    // FIXME leaking the void here a bit
                    let mut func = registry.register(Void);

                    'overloadLoop:
                    for overload in overloads.iter() {
                        if let AstType::FunctionType {name, params, ..} = overload.get(registry) {
                            if params.len() != resolved_args.len() {
                                continue;
                            }

                            for i in 0..resolved_args.len() {
                                if params[i] != resolved_args[i].get_type() {
                                    continue 'overloadLoop;
                                }
                            }

                            func = *overload;
                            break;
                        } else {
                            panic!();
                        }
                    }



                    if let AstType::FunctionType {return_type, ..} = func.get(registry) {
                        resolved_func.set_type(registry, func.get(registry));

                        return CallExpr {
                            t: return_type,
                            func: resolved_func.boxed(),
                            args: resolved_args
                        };
                    }
                }

                // calling constructor of a struct
                if let AstType::StructType {..} = resolved_func.get_type().get(registry) {
                    let mut resolved_args = vec![];

                    for arg in args {
                        resolved_args.push(arg.resolve_type(registry, symbol_table));
                    }

                    return CallExpr {
                        t: resolved_func.get_type(),
                        func: resolved_func.boxed(),
                        args: resolved_args
                    };
                }

                panic!("Calling something else than a function or a struct constructor! {:?} {:?}", func, resolved_func);
            }
        }
    }

}

impl TypedExpr {
    pub fn get_type(&self) -> TypeEntry {
        match self {
            NullLiteralExpr(t) => *t,
            BoolLiteralExpr(t, ..) => *t,
            IntLiteralExpr(t, ..) => *t,
            FloatLiteralExpr(t, ..) => *t,
            StringLiteralExpr(t, ..) => *t,
            IdentifierExpr(t, _) => *t,
            NullableExpr(t,_) => *t,
            IncrementExpr(t, _) => *t,
            DecrementExpr(t, _) => *t,
            PrefixExpr {t, .. } => *t,
            BinaryExpr {t, .. } => *t,
            AssignExpr {t, .. } => *t,
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
            if let NullLiteralExpr(t) = self {
                t.mutate(registry, NullableType {underlying});

                return;
            }

            // set underlying type
            self.set_type(registry, underlying.get(registry));

            // make the underlying type mutable
            self.get_type().mutate(registry, NullableType {underlying});

            // make expression mutable
            *self = NullableExpr(self.get_type(),self.clone().boxed());

            return;
        }


        match self {
            BoolLiteralExpr(..) => panic!(),
            FloatLiteralExpr(..) => panic!("{typ:?}"),
            StringLiteralExpr(..) => panic!(),
            IntLiteralExpr(t, v) => {
                t.mutate(registry, Float);
                
                *self = FloatLiteralExpr(*t, *v as f32)
            },
            NullLiteralExpr(t) => {
                if let NullableType {..} = typ {
                    t.mutate(registry, typ);
                } else {
                    panic!("was set to {typ:?}")
                }
            }
            IdentifierExpr(t, _) |
            NullableExpr(t, _) |
            IncrementExpr(t, _) |
            DecrementExpr(t, _) |
            PrefixExpr {t, .. } |
            BinaryExpr {t, .. } |
            AssignExpr {t, .. } |
            TupleExpr {t, .. } |
            MemberExpr {t, .. } |
            CallExpr {t, .. } => {
                t.mutate(registry, typ);
            }
        }
    }
}

pub fn bool(b: bool) -> UntypedExpr {
    BoolLiteralExpr((), b)
}

pub fn int(i: i32) -> UntypedExpr {
    IntLiteralExpr((), i)
}

pub fn float(f: f32) -> UntypedExpr {
    FloatLiteralExpr((), f)
}

pub fn string(s: String) -> UntypedExpr {
    StringLiteralExpr((), s)
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
    let binary = |op| {
        assign(assignee.clone(), Assign, binary(assignee.clone(),op,value.clone()))
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
