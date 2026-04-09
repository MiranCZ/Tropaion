use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::expression::{Expression, TypedExpr};
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, Statement, StatementBlock, TypedStmt};
use crate::ast::walking::visitor::Visitor;
use crate::error::analysis_error::AnalysisError;
use crate::error::analysis_error::AnalysisError::{IllegalThis, MultipleThisCall, ThisCallExpected};
use crate::error::context::{ErrorContext, Span};
use crate::util::spanned::Spanned;

pub struct ThisValidator<'a> {
    registry: &'a TypeRegistry,
    errors: Vec<ErrorContext<AnalysisError>>
}

impl <'a> ThisValidator<'a> {


    pub fn collect_errors(stmt: &TypedStmt, registry: &'a TypeRegistry) -> Vec<ErrorContext<AnalysisError>>{
        let mut new = ThisValidator{registry, errors: vec![]};

        stmt.walk_visit(&mut new);

        new.errors
    }

}


impl <'a> Visitor<'a> for ThisValidator<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        panic!("immutable")
    }


    fn visit_type(&mut self, typ: &TypeEntry) {
    }

    fn visit_call(&mut self, t: &TypeEntry, func: &TypedExpr, args: &[TypedExpr], span: Span) {
        if let Expression::IdentifierExpr(_, str) = &func.node {
            if *str == "this" {
                self.errors.push(ErrorContext::of(IllegalThis, span));
            }
        }
    }

    fn visit_constructor(&mut self, modifier: &Modifier, params: &Vec<Parameter>, body: &StatementBlock<TypeEntry>, span: Span) {
        let mut this_calls = vec![];

        for b in body {
            match &b.node {
                Statement::ExpressionStmt(e) => {
                    let mut checked = false;
                    if let Expression::CallExpr{func, ..} = &e.node {
                        if let Expression::IdentifierExpr(_, str) = &func.node {
                            checked = true;
                            if *str == "this" {
                                this_calls.push(e);
                            }
                        }
                    }

                    if !checked {
                        b.walk_visit(self);
                    }
                },
                _ => {
                    b.walk_visit(self);
                }
            }
        }

        if this_calls.is_empty() {
            self.errors.push(ErrorContext::of(ThisCallExpected, span));
        } else if this_calls.len() > 1 {
            for c in this_calls {
                self.errors.push(ErrorContext::of(MultipleThisCall, c.span));
            }
        }
    }


}