use std::env;
use std::fs;
use std::path::Path;
use crate::BorrowType::BorrowMut;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let dest_path = Path::new(&out_dir).join("visitor.rs");
    fs::write(&dest_path, generate_ast_walk("Visitor", "visit", BorrowMut)).unwrap();

    let dest_path = Path::new(&out_dir).join("visitor_mut.rs");
    fs::write(&dest_path, generate_ast_walk("VisitorMut", "visit_mut", BorrowMut)).unwrap();

    println!("cargo::rerun-if-changed=build.rs");
}


enum BorrowType {
    Borrow,
    BorrowMut,
    Own
}

fn generate_ast_walk(trait_name: &str, suffix: &str, borrow_type: BorrowType) -> String {
    let borrow = match borrow_type {
        BorrowType::Borrow => "&",
        BorrowType::BorrowMut => "&mut ",
        BorrowType::Own => ""
    };

    let deref = match borrow_type {
        BorrowType::Borrow => ".as_deref()",
        BorrowMut => ".as_deref_mut()",
        BorrowType::Own => ""
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
use std::collections::HashMap;

pub trait {trait_name}<'a> where Self: Sized {{

    fn get_registry<'b>(&'b self) -> &'b TypeRegistry;
    fn get_registry_mut<'b>(&'b mut self) -> &'b mut TypeRegistry;

    fn {suffix}_block(&mut self, body: {borrow}StatementBlock<TypeEntry>) {{
        for s in body {{ s.walk_{suffix}(self); }}
    }}

    fn {suffix}_expression_stmt(&mut self, expr: {borrow}TypedExpr) {{
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_var_declaration(&mut self, name: {borrow}String, is_const: {borrow} bool, value: {borrow}TypedExpr, explicit_type: {borrow}Option<TypeEntry>) {{
        self.{suffix}_expr(value);

        if let Some(t) = explicit_type {{
            t.get(self.get_registry()).walk_{suffix}(self);
        }}
    }}

    fn {suffix}_if(&mut self, condition: {borrow}TypedExpr, body: {borrow}StatementBlock<TypeEntry>, else_branch: {borrow} Option<Box<Spanned<Statement<TypeEntry>>>>) {{
        self.{suffix}_expr(condition);
        self.{suffix}_block(body);
        if let Some(branch) = else_branch {{
            branch.walk_{suffix}(self);
        }}
    }}

    fn {suffix}_while(&mut self, condition: {borrow}TypedExpr, body: {borrow}StatementBlock<TypeEntry>) {{
        self.{suffix}_expr(condition);
        self.{suffix}_block(body);
    }}

    fn {suffix}_function(&mut self, name: {borrow}String, params: {borrow}[Parameter], return_type: {borrow} TypeEntry, body: {borrow}StatementBlock<TypeEntry>) {{
        self.{suffix}_block(body);

        for p in params {{
            p.param_type.get(self.get_registry()).walk_{suffix}(self);
        }}

        return_type.get(self.get_registry()).walk_{suffix}(self);
    }}

    fn {suffix}_struct(&mut self, name: {borrow}String, fields: {borrow}[Parameter], body: {borrow}StatementBlock<TypeEntry>) {{
        self.{suffix}_block(body);
    }}

    fn {suffix}_return(&mut self, expr: {borrow}TypedExpr) {{
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_comment(&mut self, _comment: {borrow}str) {{}}

    fn {suffix}_multiline_comment(&mut self, _comment: {borrow}str) {{}}


    // Expressions

    fn {suffix}_expr(&mut self, expr: {borrow}TypedExpr) {{
        expr.walk_{suffix}(self);
    }}

    fn {suffix}_null_literal(&mut self, _t: {borrow} TypeEntry) {{}}

    fn {suffix}_bool_literal(&mut self, _t: {borrow} TypeEntry, _value: {borrow} bool) {{}}

    fn {suffix}_int_literal(&mut self, _t: {borrow} TypeEntry, _value: {borrow} i64) {{}}

    fn {suffix}_float_literal(&mut self, _t: {borrow} TypeEntry, _value: {borrow} f32) {{}}

    fn {suffix}_string_literal(&mut self, _t: {borrow} TypeEntry, _value: {borrow}str) {{}}

    fn {suffix}_identifier(&mut self, _t: {borrow} TypeEntry, _name: {borrow}str) {{}}

    fn {suffix}_array_literal(&mut self, t: {borrow} TypeEntry, values: {borrow}[TypedExpr]) {{
        for v in values {{ self.{suffix}_expr(v); }}
    }}

    fn {suffix}_nullable_expr(&mut self, t: {borrow} TypeEntry, inner: {borrow}TypedExpr) {{
        self.{suffix}_expr(inner);
    }}

    fn {suffix}_increment(&mut self, t: {borrow} TypeEntry, expr: {borrow}TypedExpr) {{
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_decrement(&mut self, t: {borrow} TypeEntry, expr: {borrow}TypedExpr) {{
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_null_deref(&mut self, t: {borrow} TypeEntry, expr: {borrow}TypedExpr) {{
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_prefix(&mut self, t: {borrow} TypeEntry, operator: {borrow} SimpleToken, expr: {borrow}TypedExpr) {{
        self.{suffix}_expr(expr);
    }}

    fn {suffix}_binary(&mut self, t: {borrow} TypeEntry, left: {borrow}TypedExpr, operator: {borrow} SimpleToken, right: {borrow}TypedExpr) {{
        self.{suffix}_expr(left);
        self.{suffix}_expr(right);
    }}

    fn {suffix}_assign(&mut self, t: {borrow} TypeEntry, assignee: {borrow}TypedExpr, value: {borrow}TypedExpr) {{
        self.{suffix}_expr(assignee);
        self.{suffix}_expr(value);
    }}

    fn {suffix}_tuple(&mut self, t: {borrow} TypeEntry, values: {borrow}[TypedExpr]) {{
        for v in values {{ self.{suffix}_expr(v); }}
    }}

    fn {suffix}_array_access(&mut self, t: {borrow} TypeEntry, property: {borrow}TypedExpr, index: {borrow}TypedExpr) {{
        self.{suffix}_expr(property);
        self.{suffix}_expr(index);
    }}

    fn {suffix}_member(&mut self, t: {borrow} TypeEntry, member: {borrow}TypedExpr, property: {borrow}TypedExpr, null_safe: {borrow} bool) {{
        self.{suffix}_expr(member);
        self.{suffix}_expr(property);
    }}

    fn {suffix}_call(&mut self, t: {borrow} TypeEntry, func: {borrow}TypedExpr, args: {borrow}[TypedExpr]) {{
        self.{suffix}_expr(func);
        for a in args {{ self.{suffix}_expr(a); }}
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

    fn {suffix}_symbol_type(&mut self, name: {borrow} String) {{
    }}

    fn {suffix}_reference_type(&mut self, underlying: {borrow} TypeEntry) {{
        underlying.get(self.get_registry()).walk_{suffix}(self)
    }}

    fn {suffix}_nullable_type(&mut self, underlying: {borrow} TypeEntry) {{
        underlying.get(self.get_registry()).walk_{suffix}(self)
    }}

    fn {suffix}_array_type(&mut self, underlying: {borrow} TypeEntry) {{
        underlying.get(self.get_registry()).walk_{suffix}(self)
    }}

    fn {suffix}_tuple_type(&mut self, types: {borrow} Vec<TypeEntry>) {{
        for t in types {{ t.get(self.get_registry()).walk_{suffix}(self) }}
    }}

    fn {suffix}_functions_type(&mut self, name: {borrow} String, overloads: {borrow} Vec<TypeEntry>) {{
        for t in overloads {{ t.get(self.get_registry()).walk_{suffix}(self) }}
    }}

    fn {suffix}_function_type(&mut self, name: {borrow} String, params: {borrow} Vec<TypeEntry>, return_type: {borrow} TypeEntry) {{
        for t in params {{ t.get(self.get_registry()).walk_{suffix}(self) }}

        return_type.get(self.get_registry()).walk_{suffix}(self);
    }}

    fn {suffix}_struct_type(&mut self, name: {borrow} String, fields: {borrow} Vec<MemberInfo>, children: {borrow} HashMap<String, MemberInfo>) {{
        for f in fields {{
            f.0.get(self.get_registry()).walk_{suffix}(self);
        }}
        for c in children.values() {{
            c.0.get(self.get_registry()).walk_{suffix}(self);
        }}
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
                visitor.{suffix}_var_declaration(name, is_const, value, explicit_type);
            }}
            Statement::IfStmt {{ condition, body, else_branch }} => {{
                visitor.{suffix}_if(condition, body, else_branch);
            }}
            Statement::WhileStmt {{ condition, body }} => {{
                visitor.{suffix}_while(condition, body);
            }}
            Statement::FunctionStmt {{ name, params, return_type, body }} => {{
                visitor.{suffix}_function(name, params, return_type, body);
            }}
            Statement::StructStmt {{ name, fields, body }} => {{
                visitor.{suffix}_struct(name, fields, body);
            }}
            Statement::ReturnStmt(expr) => {{
                visitor.{suffix}_return(expr);
            }}
            Statement::CommentStmt(s) => {{
                visitor.{suffix}_comment(s);
            }}
            Statement::MultilineCommentStmt(s) => {{
                visitor.{suffix}_multiline_comment(s);
            }}
        }}
    }}

}}

impl TypedExpr {{

    pub fn walk_{suffix}<'a>({borrow}self, visitor: &mut impl {trait_name}<'a>) {{
        match {borrow}self.node {{
            Expression::NullLiteralExpr(t) => {{
                visitor.{suffix}_null_literal(t);
            }}
            Expression::BoolLiteralExpr(t, v) => {{
                visitor.{suffix}_bool_literal(t, v);
            }}
            Expression::IntLiteralExpr(t, v) => {{
                visitor.{suffix}_int_literal(t, v);
            }}
            Expression::FloatLiteralExpr(t, v) => {{
                visitor.{suffix}_float_literal(t, v);
            }}
            Expression::StringLiteralExpr(t, v) => {{
                visitor.{suffix}_string_literal(t, v);
            }}
            Expression::IdentifierExpr(t, name) => {{
                visitor.{suffix}_identifier(t, name);
            }}
            Expression::ArrayLiteralExpr(t, values) => {{
                visitor.{suffix}_array_literal(t, values);
            }}
            Expression::NullableExpr(t, inner) => {{
                visitor.{suffix}_nullable_expr(t, inner);
            }}
            Expression::IncrementExpr(t, expr) => {{
                visitor.{suffix}_increment(t, expr);
            }}
            Expression::DecrementExpr(t, expr) => {{
                visitor.{suffix}_decrement(t, expr);
            }}
            Expression::NullDerefExpr(t, expr) => {{
                visitor.{suffix}_null_deref(t, expr);
            }}
            Expression::PrefixExpr {{ t, operator, expr }} => {{
                visitor.{suffix}_prefix(t, operator, expr);
            }}
            Expression::BinaryExpr {{ t, left, operator, right }} => {{
                visitor.{suffix}_binary(t, left, operator, right);
            }}
            Expression::AssignExpr {{ t, assignee, value }} => {{
                visitor.{suffix}_assign(t, assignee, value);
            }}
            Expression::TupleExpr {{ t, values }} => {{
                visitor.{suffix}_tuple(t, values);
            }}
            Expression::ArrayAccessExpr {{ t, property, index }} => {{
                visitor.{suffix}_array_access(t, property, index);
            }}
            Expression::MemberExpr {{ t, member, property, null_safe }} => {{
                visitor.{suffix}_member(t, member, property, null_safe);
            }}
            Expression::CallExpr {{ t, func, args }} => {{
                visitor.{suffix}_call(t, func, args);
            }}
        }}
    }}

}}


impl AstType {{
    pub fn walk_{suffix}<'a>({borrow}self, visitor: &mut impl {trait_name}<'a>) {{
        match self {{
            AstType::UnknownType => visitor.{suffix}_unknown_type(),
            AstType::Void => visitor.{suffix}_void_type(),
            AstType::Bool => visitor.{suffix}_bool_type(),
            AstType::Int => visitor.{suffix}_int_type(),
            AstType::Float => visitor.{suffix}_float_type(),
            AstType::StringType => visitor.{suffix}_string_type(),
            AstType::SymbolType(name) => visitor.{suffix}_symbol_type(name),
            AstType::ReferenceType {{ underlying }} => visitor.{suffix}_reference_type(underlying),
            AstType::NullableType {{ underlying }} => visitor.{suffix}_nullable_type(underlying),
            AstType::ArrayType {{ underlying }} => visitor.{suffix}_array_type(underlying),
            AstType::TupleType(types) => visitor.{suffix}_tuple_type(types),
            AstType::FunctionsType {{ name, overloads }} => visitor.{suffix}_functions_type(name, overloads),
            AstType::FunctionType {{ name, params, return_type }} => visitor.{suffix}_function_type(name, params, return_type),
            AstType::StructType {{ name, fields, children }} => visitor.{suffix}_struct_type(name, fields, children),
        }}
    }}
}}

"#);
}