use std::fmt::Write;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::expression::Expression;
use crate::ast::statement::{Parameter, Statement};
use crate::util::spanned::Spanned;

/// ---- THIS WHOLE FILE IS AI-GENERATED ------
///
///
/// A helper trait to generically format types if they exist on the AST nodes.
pub trait TypeInfo {
    fn get_type_string(&self, registry: Option<&TypeRegistry>) -> String;
}

// Untyped ASTs `()` produce no type string
impl TypeInfo for () {
    fn get_type_string(&self, _registry: Option<&TypeRegistry>) -> String {
        String::new()
    }
}

// Typed ASTs `TypeEntry` format their actual resolved type
impl TypeInfo for TypeEntry {
    fn get_type_string(&self, registry: Option<&TypeRegistry>) -> String {
        if let Some(reg) = registry {
            format!(" [type: {}]", self.format(reg))
        } else {
            " [type: ?]".to_string()
        }
    }
}

/// A utility to format the AST structure and its types for debugging.
pub struct AstPrinter<'a> {
    registry: Option<&'a TypeRegistry>,
}

impl<'a> AstPrinter<'a> {
    /// Create a new printer. Pass `Some(&registry)` for typed ASTs, or `None` for untyped.
    pub fn new(registry: Option<&'a TypeRegistry>) -> Self {
        Self { registry }
    }

    /// Prints a full statement tree to a formatted String.
    pub fn print_statement<T: TypeInfo>(&self, stmt: &Spanned<Statement<T>>) -> String {
        let mut out = String::new();
        self.fmt_stmt(&stmt.node, 0, &mut out);
        out
    }

    /// Prints an isolated expression tree to a formatted String.
    pub fn print_expression<T: TypeInfo>(&self, expr: &Spanned<Expression<T>>) -> String {
        let mut out = String::new();
        self.fmt_expr(&expr.node, 0, &mut out);
        out
    }

    // --- Internal Formatting Logic ---

    fn ind(&self, indent: usize) -> String {
        "  ".repeat(indent)
    }

    fn fmt_stmt<T: TypeInfo>(&self, stmt: &Statement<T>, indent: usize, out: &mut String) {
        let prefix = self.ind(indent);

        match stmt {
            Statement::BlockStmt { body } => {
                writeln!(out, "{prefix}BlockStmt").unwrap();
                for b in body {
                    self.fmt_stmt(&b.node, indent + 1, out);
                }
            }
            Statement::ExpressionStmt(expr) => {
                writeln!(out, "{prefix}ExpressionStmt").unwrap();
                self.fmt_expr(&expr.node, indent + 1, out);
            }
            Statement::VarDeclarationStmt { name, is_const, value, explicit_type } => {
                let const_str = if *is_const { "const " } else { "var " };
                let type_str = explicit_type.as_ref().map_or(String::new(), |typ| {
                    self.registry.map_or_else(|| ": ?".to_string(), |r| format!(": {}", typ.format(r)))
                });

                writeln!(out, "{prefix}VarDeclaration ({const_str}{name}{type_str})").unwrap();
                self.fmt_expr(&value.node, indent + 1, out);
            }
            Statement::IfStmt { condition, body, else_branch } => {
                writeln!(out, "{prefix}IfStmt").unwrap();
                writeln!(out, "{}Condition:", self.ind(indent + 1)).unwrap();
                self.fmt_expr(&condition.node, indent + 2, out);

                writeln!(out, "{}Body:", self.ind(indent + 1)).unwrap();
                for b in body {
                    self.fmt_stmt(&b.node, indent + 2, out);
                }

                if let Some(else_b) = else_branch {
                    writeln!(out, "{}Else:", self.ind(indent + 1)).unwrap();
                    self.fmt_stmt(&else_b.node, indent + 2, out);
                }
            }
            Statement::WhileStmt { condition, body } => {
                writeln!(out, "{prefix}WhileStmt").unwrap();
                writeln!(out, "{}Condition:", self.ind(indent + 1)).unwrap();
                self.fmt_expr(&condition.node, indent + 2, out);

                writeln!(out, "{}Body:", self.ind(indent + 1)).unwrap();
                for b in body {
                    self.fmt_stmt(&b.node, indent + 2, out);
                }
            }
            Statement::FunctionStmt { name, modifier, generics, params, return_type, body } => {
                let gen_str = if generics.is_empty() { String::new() } else { format!("<{}>", generics.join(", ")) };
                let ret_str = self.registry.map_or("?".to_string(), |r| return_type.format(r));

                writeln!(out, "{prefix}FunctionStmt {name}{gen_str} -> {ret_str} ({modifier:?})").unwrap();
                self.fmt_params(params, indent + 1, out);

                writeln!(out, "{}Body:", self.ind(indent + 1)).unwrap();
                for b in body {
                    self.fmt_stmt(&b.node, indent + 2, out);
                }
            }
            Statement::ConstructorStmt { modifier, params, body } => {
                writeln!(out, "{prefix}ConstructorStmt ({modifier:?})").unwrap();
                self.fmt_params(params, indent + 1, out);

                writeln!(out, "{}Body:", self.ind(indent + 1)).unwrap();
                for b in body {
                    self.fmt_stmt(&b.node, indent + 2, out);
                }
            }
            Statement::StructStmt { name, public_constructor, fields, body, generics } => {
                let gen_str = if generics.is_empty() { String::new() } else { format!("<{}>", generics.join(", ")) };
                writeln!(out, "{prefix}StructStmt {name}{gen_str} (pub_ctor: {public_constructor})").unwrap();

                self.fmt_params(fields, indent + 1, out);
                writeln!(out, "{}Body:", self.ind(indent + 1)).unwrap();
                for b in body {
                    self.fmt_stmt(&b.node, indent + 2, out);
                }
            }
            Statement::EnumStmt { name, values, body } => {
                writeln!(out, "{prefix}EnumStmt {name}").unwrap();
                for v in values {
                    writeln!(out, "{}- {v}", self.ind(indent + 1)).unwrap();
                }

                writeln!(out, "{}Body:", self.ind(indent + 1)).unwrap();
                for b in body {
                    self.fmt_stmt(&b.node, indent + 2, out);
                }
            }
            Statement::ReturnStmt(expr) => {
                writeln!(out, "{prefix}ReturnStmt").unwrap();
                if let Some(e) = &expr {
                    self.fmt_expr(&e.node, indent + 1, out);
                }
            }
            Statement::LoopInterrupt { break_loop } => {
                let action = if *break_loop { "break" } else { "continue" };
                writeln!(out, "{prefix}LoopInterrupt ({action})").unwrap();
            }
            Statement::CommentStmt(c) => writeln!(out, "{prefix}// {c}").unwrap(),
            Statement::MultilineCommentStmt(c) => writeln!(out, "{prefix}/* {c} */").unwrap(),
        }
    }

    fn fmt_params(&self, params: &[Parameter], indent: usize, out: &mut String) {
        if params.is_empty() { return; }
        writeln!(out, "{}Parameters:", self.ind(indent)).unwrap();
        for p in params {
            let typ_str = self.registry.map_or("?".to_string(), |r| p.param_type.format(r));
            writeln!(out, "{}- {}: {}", self.ind(indent + 1), p.name, typ_str).unwrap();
        }
    }

    fn fmt_expr<T: TypeInfo>(&self, expr: &Expression<T>, indent: usize, out: &mut String) {
        let prefix = self.ind(indent);

        // Helper to grab the type string automatically
        let type_label = |t: &T| t.get_type_string(self.registry);

        match expr {
            Expression::ErroredExpr(t) => writeln!(out, "{prefix}ErroredExpr{}", type_label(t)).unwrap(),
            Expression::NullLiteralExpr(t) => writeln!(out, "{prefix}NullLiteral{}", type_label(t)).unwrap(),
            Expression::BoolLiteralExpr(t, v) => writeln!(out, "{prefix}BoolLiteral({v}){}", type_label(t)).unwrap(),
            Expression::IntLiteralExpr(t, v) => writeln!(out, "{prefix}IntLiteral({v}){}", type_label(t)).unwrap(),
            Expression::FloatLiteralExpr(t, v) => writeln!(out, "{prefix}FloatLiteral({v}){}", type_label(t)).unwrap(),
            Expression::StringLiteralExpr(t, v) => writeln!(out, "{prefix}StringLiteral({v:?}){}", type_label(t)).unwrap(),

            Expression::ArrayLiteralExpr(t, vals) => {
                writeln!(out, "{prefix}ArrayLiteral{}", type_label(t)).unwrap();
                for v in vals {
                    self.fmt_expr(&v.node, indent + 1, out);
                }
            }
            Expression::IdentifierExpr(t, name) => writeln!(out, "{prefix}Identifier({name}){}", type_label(t)).unwrap(),

            Expression::NullableExpr(t, e) => {
                writeln!(out, "{prefix}NullableExpr{}", type_label(t)).unwrap();
                self.fmt_expr(&e.node, indent + 1, out);
            }
            Expression::IncrementExpr(t, e) => {
                writeln!(out, "{prefix}IncrementExpr{}", type_label(t)).unwrap();
                self.fmt_expr(&e.node, indent + 1, out);
            }
            Expression::DecrementExpr(t, e) => {
                writeln!(out, "{prefix}DecrementExpr{}", type_label(t)).unwrap();
                self.fmt_expr(&e.node, indent + 1, out);
            }
            Expression::NullDerefExpr(t, e) => {
                writeln!(out, "{prefix}NullDerefExpr{}", type_label(t)).unwrap();
                self.fmt_expr(&e.node, indent + 1, out);
            }

            Expression::PrefixExpr { t, operator, expr } => {
                writeln!(out, "{prefix}PrefixExpr({operator:?}){}", type_label(t)).unwrap();
                self.fmt_expr(&expr.node, indent + 1, out);
            }
            Expression::BinaryExpr { t, left, operator, right } => {
                writeln!(out, "{prefix}BinaryExpr({operator:?}){}", type_label(t)).unwrap();
                self.fmt_expr(&left.node, indent + 1, out);
                self.fmt_expr(&right.node, indent + 1, out);
            }
            Expression::AssignExpr { t, assignee, value } => {
                writeln!(out, "{prefix}AssignExpr{}", type_label(t)).unwrap();
                self.fmt_expr(&assignee.node, indent + 1, out);
                self.fmt_expr(&value.node, indent + 1, out);
            }

            Expression::TupleExpr { t, values } => {
                writeln!(out, "{prefix}TupleExpr{}", type_label(t)).unwrap();
                for v in values {
                    self.fmt_expr(&v.node, indent + 1, out);
                }
            }
            Expression::ArrayAccessExpr { t, property, index } => {
                writeln!(out, "{prefix}ArrayAccessExpr{}", type_label(t)).unwrap();
                self.fmt_expr(&property.node, indent + 1, out);
                self.fmt_expr(&index.node, indent + 1, out);
            }
            Expression::MemberExpr { t, member, property, null_safe } => {
                writeln!(out, "{prefix}MemberExpr(null_safe: {null_safe}){}", type_label(t)).unwrap();
                self.fmt_expr(&member.node, indent + 1, out);
                self.fmt_expr(&property.node, indent + 1, out);
            }

            Expression::CallExpr { t, func, args } => {
                writeln!(out, "{prefix}CallExpr{}", type_label(t)).unwrap();
                writeln!(out, "{}Function:", self.ind(indent + 1)).unwrap();
                self.fmt_expr(&func.node, indent + 2, out);

                if !args.is_empty() {
                    writeln!(out, "{}Arguments:", self.ind(indent + 1)).unwrap();
                    for a in args {
                        self.fmt_expr(&a.node, indent + 2, out);
                    }
                }
            }
        }
    }
}