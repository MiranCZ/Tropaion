use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::expression::{Expression, UntypedExpr};
use crate::ast::statement::{Statement, UntypedStmt};
use crate::error::parser_error::ParserError;
use crate::parser::binding_power::Bp;
use crate::parser::Parser;


pub type ReturnedStatement = Result<UntypedStmt, ParserError>;
pub type ReturnedExpression = Result<UntypedExpr, ParserError>;
pub type ReturnedType = Result<TypeEntry, ParserError>;


pub type StatementHandler = fn(&mut TypeRegistry,&mut Parser) -> ReturnedStatement;
pub type NudHandler = fn(&mut TypeRegistry,&mut Parser) -> ReturnedExpression;
pub type LedHandler = fn(&mut TypeRegistry,&mut Parser, UntypedExpr, u32) -> ReturnedExpression;

pub type TypeNudHandler = fn(&mut TypeRegistry, &mut Parser) -> ReturnedType;
pub type TypeLedHandler = fn(&mut TypeRegistry,&mut Parser, TypeEntry, u32) -> ReturnedType;


#[derive(Clone, Copy)]
pub struct LedInfo {
    pub handler: LedHandler,
    pub rbp: Bp,
    pub lbp: Bp
}

#[derive(Clone, Copy)]
pub struct TypeLedInfo {
    pub type_handler: TypeLedHandler,
    pub rbp: Bp,
    pub lbp: Bp
}