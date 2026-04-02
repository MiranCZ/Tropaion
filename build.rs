use std::env;
use std::fs;
use std::path::Path;
use crate::BorrowType::{Borrow, BorrowMut};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("visitor.rs");
    fs::write(&dest_path, generate_ast_walk("Visitor", "visit", Borrow)).unwrap();

    let dest_path = Path::new(&out_dir).join("visitor_mut.rs");
    fs::write(&dest_path, generate_ast_walk("VisitorMut", "visit_mut", BorrowMut)).unwrap();

    println!("cargo::rerun-if-changed=build.rs");
}


enum BorrowType {
    Borrow,
    BorrowMut
}

fn generate_ast_walk(trait_name: &str, suffix: &str, borrow_type: BorrowType) -> String {
    let borrow = match borrow_type {
        BorrowType::Borrow => "&",
        BorrowType::BorrowMut => "&mut ",
    };

    let mut_suffix = match borrow_type {
        Borrow => "",
        BorrowMut => "_mut",
    };

    let type_entry_part =
        match borrow_type {
            BorrowType::Borrow => {r#"
impl TypeEntry {

    pub fn walk_visit<'a>(&self, visitor: &mut impl Visitor<'a>) {
        let typ = self.get(visitor.get_registry());

        typ.walk_visit(visitor);
    }
}
    "#},
            BorrowMut => {r#"
impl TypeEntry {

    pub fn walk_visit_mut<'a>(&mut self, visitor: &mut impl VisitorMut<'a>) {
        let mut typ = self.get(visitor.get_registry());

        typ.walk_visit_mut(visitor);

        self.mutate(visitor.get_registry_mut(), typ);
    }
}
    "#}
        };

    return format!(r#"


use crate::lexer::token::SimpleToken;
use crate::ast::statement::Statement;
use crate::ast::statement::Parameter;
use crate::ast::statement::StatementBlock;
use crate::ast::statement::TypedStmt;
use crate::ast::expression::Expression;
use crate::ast::expression::TypedExpr;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::MemberInfo;
use crate::analysis::type_registry::TypeEntry;
use crate::analysis::type_registry::TypeRegistry;
use crate::util::spanned::Spanned;
use crate::error::context::Span;
use std::collections::HashMap;
use ordermap::OrderMap;

pub trait {trait_name}<'a> where Self: Sized {{

    fn get_registry<'b>(&'b self) -> &'b TypeRegistry;
    fn get_registry_mut<'b>(&'b mut self) -> &'b mut TypeRegistry;

    fn {suffix}_block(&mut self, body: {borrow}StatementBlock<TypeEntry>) {{
        for s in body {{ s.walk_{suffix}(self); }}
    }}

    fn {suffix}_expression_stmt(&mut self, expr: {borrow}TypedExpr) {{
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_var_declaration(&mut self, name: {borrow}String, is_const: {borrow} bool, value: {borrow}TypedExpr, explicit_type: {borrow}Option<TypeEntry>, span: Span) {{
        self.{suffix}_expr(value);

        if let Some(t) = explicit_type {{
            self.{suffix}_type(t);
        }}
    }}

    fn {suffix}_if(&mut self, condition: {borrow}TypedExpr, body: {borrow}StatementBlock<TypeEntry>, else_branch: {borrow} Option<Box<Spanned<Statement<TypeEntry>>>>, span: Span) {{
        self.{suffix}_expr(condition);
        self.{suffix}_block(body);
        if let Some(branch) = else_branch {{
            branch.walk_{suffix}(self);
        }}
    }}

    fn {suffix}_while(&mut self, condition: {borrow}TypedExpr, body: {borrow}StatementBlock<TypeEntry>, span: Span) {{
        self.{suffix}_expr(condition);
        self.{suffix}_block(body);
    }}

    fn {suffix}_function(&mut self, name: {borrow}String, generics: {borrow} Vec<String>, params: {borrow}Vec<Parameter>, return_type: {borrow} TypeEntry, body: {borrow}StatementBlock<TypeEntry>, span: Span) {{
        self.{suffix}_block(body);

        for p in params {{
            self.{suffix}_type({borrow} p.param_type);
        }}

        self.{suffix}_type(return_type);
    }}

    fn {suffix}_struct(&mut self, name: {borrow}String, fields: {borrow}Vec<Parameter>, body: {borrow}StatementBlock<TypeEntry>, generics: {borrow} Vec<String>, span: Span) {{
        self.{suffix}_block(body);
    }}

    fn {suffix}_enum(&mut self, name: {borrow} String, values: {borrow} Vec<String>, body: {borrow} StatementBlock<TypeEntry>, span: Span) {{
        self.{suffix}_block(body);
    }}

    fn {suffix}_return(&mut self, expr: {borrow}TypedExpr, span: Span) {{
        self.{suffix}_expr(expr);
    }}
    
    fn {suffix}_loop_interrupt(&mut self, break_loop: {borrow}bool, span: Span) {{
    }}

    fn {suffix}_comment(&mut self, _comment: {borrow}String, span: Span) {{}}

    fn {suffix}_multiline_comment(&mut self, _comment: {borrow}String, span: Span) {{}}


    // Expressions

    fn {suffix}_expr(&mut self, expr: {borrow}TypedExpr) {{
        expr.walk_{suffix}(self);
    }}

    fn {suffix}_errored(&mut self, t: {borrow} TypeEntry, span: Span) {{
        self.{suffix}_type(t);
    }}

    fn {suffix}_null_literal(&mut self, t: {borrow} TypeEntry, span: Span) {{
        self.{suffix}_type(t); 
    }}

    fn {suffix}_bool_literal(&mut self, t: {borrow} TypeEntry, _value: {borrow} bool, span: Span) {{
        self.{suffix}_type(t); 
    }}

    fn {suffix}_int_literal(&mut self, t: {borrow} TypeEntry, _value: {borrow} i64, span: Span) {{
        self.{suffix}_type(t); 
    }}

    fn {suffix}_float_literal(&mut self, t: {borrow} TypeEntry, _value: {borrow} f32, span: Span) {{
        self.{suffix}_type(t); 
    }}

    fn {suffix}_string_literal(&mut self, t: {borrow} TypeEntry, _value: {borrow}String, span: Span) {{
        self.{suffix}_type(t); 
    }}

    fn {suffix}_identifier(&mut self, t: {borrow} TypeEntry, _name: {borrow}String, span: Span) {{
        self.{suffix}_type(t); 
    }}

    fn {suffix}_array_literal(&mut self, t: {borrow} TypeEntry, values: {borrow}[TypedExpr], span: Span) {{
        self.{suffix}_type(t); 
        for v in values {{ self.{suffix}_expr(v); }}
    }}

    fn {suffix}_nullable_expr(&mut self, t: {borrow} TypeEntry, inner: {borrow}TypedExpr, span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(inner);
    }}

    fn {suffix}_increment(&mut self, t: {borrow} TypeEntry, expr: {borrow}TypedExpr, span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_decrement(&mut self, t: {borrow} TypeEntry, expr: {borrow}TypedExpr, span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_null_deref(&mut self, t: {borrow} TypeEntry, expr: {borrow}TypedExpr, span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_prefix(&mut self, t: {borrow} TypeEntry, operator: {borrow} SimpleToken, expr: {borrow}TypedExpr, span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_binary(&mut self, t: {borrow} TypeEntry, left: {borrow}TypedExpr, operator: {borrow} SimpleToken, right: {borrow}TypedExpr, span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(left);
        self.{suffix}_expr(right);
    }}

    fn {suffix}_assign(&mut self, t: {borrow} TypeEntry, assignee: {borrow}TypedExpr, value: {borrow}TypedExpr, span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(assignee);
        self.{suffix}_expr(value);
    }}

    fn {suffix}_tuple(&mut self, t: {borrow} TypeEntry, values: {borrow}[TypedExpr], span: Span) {{
        self.{suffix}_type(t); 
        for v in values {{ self.{suffix}_expr(v); }}
    }}

    fn {suffix}_array_access(&mut self, t: {borrow} TypeEntry, property: {borrow}TypedExpr, index: {borrow}TypedExpr, span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(property);
        self.{suffix}_expr(index);
    }}

    fn {suffix}_member(&mut self, t: {borrow} TypeEntry, member: {borrow}TypedExpr, property: {borrow}TypedExpr, null_safe: {borrow} bool, span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(member);
        self.{suffix}_expr(property);
    }}

    fn {suffix}_call(&mut self, t: {borrow} TypeEntry, func: {borrow}TypedExpr, args: {borrow}[TypedExpr], span: Span) {{
        self.{suffix}_type(t); 
        self.{suffix}_expr(func);
        for a in args {{ self.{suffix}_expr(a); }}
    }}

    // types
    
    fn {suffix}_type(&mut self, typ: {borrow}TypeEntry) {{
        typ.walk_{suffix}(self);
    }}

    fn {suffix}_errored_type(&mut self) {{
    }}

    fn {suffix}_unknown_type(&mut self) {{
    }}

    fn {suffix}_void_type(&mut self) {{
    }}

    fn {suffix}_bool_type(&mut self) {{
    }}

    fn {suffix}_int_type(&mut self) {{
    }}

    fn {suffix}_float_type(&mut self) {{
    }}

    fn {suffix}_string_type(&mut self) {{
    }}

    fn {suffix}_symbol_type(&mut self, name: {borrow} String, generics: {borrow} Vec<TypeEntry>) {{
    }}

    fn {suffix}_reference_type(&mut self, underlying: {borrow} TypeEntry) {{
        self.{suffix}_type(underlying);
    }}

    fn {suffix}_nullable_type(&mut self, underlying: {borrow} TypeEntry) {{
        self.{suffix}_type(underlying);
    }}

    fn {suffix}_array_type(&mut self, underlying: {borrow} TypeEntry) {{
        self.{suffix}_type(underlying);
    }}

    fn {suffix}_tuple_type(&mut self, types: {borrow} Vec<TypeEntry>) {{
        for t in types {{ self.{suffix}_type(t) }}
    }}

    fn {suffix}_functions_type(&mut self, name: {borrow} String, overloads: {borrow} Vec<TypeEntry>) {{
        for t in overloads {{ self.{suffix}_type(t) }}
    }}

    fn {suffix}_function_type(&mut self, name: {borrow} String, generics: {borrow} OrderMap<String, TypeEntry>, params: {borrow} Vec<TypeEntry>, return_type: {borrow} TypeEntry) {{
        for g in generics.values{mut_suffix}() {{ self.{suffix}_type(g); }}

        for t in params {{ self.{suffix}_type(t) }}

        self.{suffix}_type(return_type);
    }}

    fn {suffix}_struct_type(&mut self, name: {borrow} String, generics: {borrow} OrderMap<String, TypeEntry>, fields: {borrow} Vec<MemberInfo>, children: {borrow} HashMap<String, MemberInfo>) {{
        for g in generics.values{mut_suffix}() {{
            self.{suffix}_type(g);
        }}
        for f in fields {{
            self.{suffix}_type({borrow} f.0);
        }}
        for c in children.values{mut_suffix}() {{
            self.{suffix}_type({borrow} c.0);
        }}
    }}

    fn {suffix}_enum_type(&mut self, name: {borrow} String, values: {borrow} Vec<String>) {{
    }}

    fn {suffix}_generic_type(&mut self, name: {borrow} String) {{
    }}

}}

impl TypedStmt {{

    pub fn walk_{suffix}<'a>({borrow}self, visitor: &mut impl {trait_name}<'a>) {{
        match {borrow}self.node {{
            Statement::BlockStmt {{ body }} => {{
                visitor.{suffix}_block(body);
            }}
            Statement::ExpressionStmt(expr) => {{
                visitor.{suffix}_expression_stmt(expr);
            }}
            Statement::VarDeclarationStmt {{ name, is_const, value, explicit_type }} => {{
                visitor.{suffix}_var_declaration(name, is_const, value, explicit_type, self.span);
            }}
            Statement::IfStmt {{ condition, body, else_branch }} => {{
                visitor.{suffix}_if(condition, body, else_branch, self.span);
            }}
            Statement::WhileStmt {{ condition, body }} => {{
                visitor.{suffix}_while(condition, body, self.span);
            }}
            Statement::FunctionStmt {{ name, generics, params, return_type, body }} => {{
                visitor.{suffix}_function(name, generics, params, return_type, body, self.span);
            }}
            Statement::StructStmt {{ name, fields, body, generics }} => {{
                visitor.{suffix}_struct(name, fields, body, generics, self.span);
            }}
            Statement::EnumStmt {{ name, values, body }} => {{
                visitor.{suffix}_enum(name, values, body, self.span);
            }}
            Statement::ReturnStmt(expr) => {{
                visitor.{suffix}_return(expr, self.span);
            }}
            Statement::LoopInterrupt {{break_loop}} => {{
                visitor.{suffix}_loop_interrupt(break_loop, self.span); 
            }}
            Statement::CommentStmt(s) => {{
                visitor.{suffix}_comment(s, self.span);
            }}
            Statement::MultilineCommentStmt(s) => {{
                visitor.{suffix}_multiline_comment(s, self.span);
            }}
        }}
    }}

}}

impl TypedExpr {{

    pub fn walk_{suffix}<'a>({borrow}self, visitor: &mut impl {trait_name}<'a>) {{
        match {borrow}self.node {{
            Expression::ErroredExpr(t) => {{
                visitor.{suffix}_errored(t, self.span);
            }}
            Expression::NullLiteralExpr(t) => {{
                visitor.{suffix}_null_literal(t, self.span);
            }}
            Expression::BoolLiteralExpr(t, v) => {{
                visitor.{suffix}_bool_literal(t, v, self.span);
            }}
            Expression::IntLiteralExpr(t, v) => {{
                visitor.{suffix}_int_literal(t, v, self.span);
            }}
            Expression::FloatLiteralExpr(t, v) => {{
                visitor.{suffix}_float_literal(t, v, self.span);
            }}
            Expression::StringLiteralExpr(t, v) => {{
                visitor.{suffix}_string_literal(t, v, self.span);
            }}
            Expression::IdentifierExpr(t, name) => {{
                visitor.{suffix}_identifier(t, name, self.span);
            }}
            Expression::ArrayLiteralExpr(t, values) => {{
                visitor.{suffix}_array_literal(t, values, self.span);
            }}
            Expression::NullableExpr(t, inner) => {{
                visitor.{suffix}_nullable_expr(t, inner, self.span);
            }}
            Expression::IncrementExpr(t, expr) => {{
                visitor.{suffix}_increment(t, expr, self.span);
            }}
            Expression::DecrementExpr(t, expr) => {{
                visitor.{suffix}_decrement(t, expr, self.span);
            }}
            Expression::NullDerefExpr(t, expr) => {{
                visitor.{suffix}_null_deref(t, expr, self.span);
            }}
            Expression::PrefixExpr {{ t, operator, expr }} => {{
                visitor.{suffix}_prefix(t, operator, expr, self.span);
            }}
            Expression::BinaryExpr {{ t, left, operator, right }} => {{
                visitor.{suffix}_binary(t, left, operator, right, self.span);
            }}
            Expression::AssignExpr {{ t, assignee, value }} => {{
                visitor.{suffix}_assign(t, assignee, value, self.span);
            }}
            Expression::TupleExpr {{ t, values }} => {{
                visitor.{suffix}_tuple(t, values, self.span);
            }}
            Expression::ArrayAccessExpr {{ t, property, index }} => {{
                visitor.{suffix}_array_access(t, property, index, self.span);
            }}
            Expression::MemberExpr {{ t, member, property, null_safe }} => {{
                visitor.{suffix}_member(t, member, property, null_safe, self.span);
            }}
            Expression::CallExpr {{ t, func, args }} => {{
                visitor.{suffix}_call(t, func, args, self.span);
            }}
        }}
    }}

}}


impl AstType {{
    pub fn walk_{suffix}<'a>({borrow}self, visitor: &mut impl {trait_name}<'a>) {{
        match self {{
            AstType::ErroredType => visitor.{suffix}_errored_type(),
            
            AstType::UnknownType => visitor.{suffix}_unknown_type(),
            AstType::Void => visitor.{suffix}_void_type(),
            AstType::Bool => visitor.{suffix}_bool_type(),
            AstType::Int => visitor.{suffix}_int_type(),
            AstType::Float => visitor.{suffix}_float_type(),
            AstType::StringType => visitor.{suffix}_string_type(),
            AstType::SymbolType{{name, generics}} => visitor.{suffix}_symbol_type(name, generics),
            AstType::ReferenceType {{ underlying }} => visitor.{suffix}_reference_type(underlying),
            AstType::NullableType {{ underlying }} => visitor.{suffix}_nullable_type(underlying),
            AstType::ArrayType {{ underlying }} => visitor.{suffix}_array_type(underlying),
            AstType::TupleType(types) => visitor.{suffix}_tuple_type(types),
            AstType::FunctionsType {{ name, overloads }} => visitor.{suffix}_functions_type(name, overloads),
            AstType::FunctionType {{ name,generics, params, return_type }} => visitor.{suffix}_function_type(name, generics, params, return_type),
            AstType::StructType {{ name, generics, fields, children }} => visitor.{suffix}_struct_type(name, generics, fields, children),
            AstType::EnumType {{name, values}} => visitor.{suffix}_enum_type(name, values),
            AstType::GenericType {{ name }} => visitor.{suffix}_generic_type(name),
        }}
    }}
}}


{type_entry_part}"#);
}