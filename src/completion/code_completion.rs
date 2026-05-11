use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::expression::TypedExpr;
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, StatementBlock, TypedStmt};
use crate::ast::walking::visitor::Visitor;
use crate::completion::completion_type::CompletionType;
use crate::error::context::Span;
use crate::intrinsics::type_injector;
use crate::lexer::token::SimpleToken;
use std::collections::HashMap;

pub fn get_code_suggestions(cursor: usize, code: &TypedStmt, registry: &mut TypeRegistry) -> HashMap<String, CompletionType> {
    let mut suggestions = HashMap::new();

    for func in type_injector::get_injected_functions(registry) {
        if let AstType::FunctionType {name, ..} = func {
            if name.starts_with("__") {
                continue
            }
            suggestions.insert(name, CompletionType::Function);
        }
    }
    for struct_t in type_injector::get_injected_structs(registry) {
        if let AstType::StructType {name, ..} = struct_t {
            if name.starts_with("__") {
                continue
            }
            suggestions.insert(name, CompletionType::Function);
        }
    }

    let mut collector = SymbolCollector::new(registry, cursor);

    code.walk_visit(&mut collector);

    for (k, v) in collector.symbols {
        suggestions.insert(k, v);
    }

    add_keywords(&mut suggestions);

    suggestions
}

fn add_keywords(suggestions: &mut HashMap<String, CompletionType>) {
    let keywords = vec![
        (SimpleToken::Const, CompletionType::KwDeclaration),
        (SimpleToken::Let, CompletionType::KwDeclaration),
        (SimpleToken::If, CompletionType::KwControl),
        (SimpleToken::Else, CompletionType::KwControl),
        (SimpleToken::While, CompletionType::KwControl),
        (SimpleToken::For, CompletionType::KwControl),
        (SimpleToken::Break, CompletionType::KwControl),
        (SimpleToken::Const, CompletionType::KwControl),
        (SimpleToken::Return, CompletionType::KwReturn),
        (SimpleToken::Struct, CompletionType::KwDefinition),
        (SimpleToken::Enum, CompletionType::KwDefinition),
        (SimpleToken::Fn, CompletionType::KwDefinition),
        (SimpleToken::Init, CompletionType::KwDefinition),
        (SimpleToken::Pub, CompletionType::KwVisibility),
        (SimpleToken::Priv, CompletionType::KwVisibility),
    ];

    for (token, compl_type) in keywords {
        suggestions.insert(token.string_representation().to_string(), compl_type);
    }
}


struct SymbolCollector<'a> {
    registry: &'a TypeRegistry,
    symbols: HashMap<String, CompletionType>,
    cursor: usize
}

impl <'a> SymbolCollector<'a> {

    pub fn new(registry: &'a TypeRegistry, cursor: usize) -> Self {
        Self {
            registry, cursor,
            symbols: HashMap::new()
        }
    }

    fn is_within_cursor(&self, span: Span) -> bool {
        span.from <= self.cursor && span.to > self.cursor
    }

}

impl <'a> Visitor<'a> for SymbolCollector<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        todo!()
    }

    fn visit_var_declaration(&mut self, name: &String, is_const: &bool, value: &TypedExpr, _explicit_type: &Option<TypeEntry>, _span: Span) {
        // declared after cursor
        if value.span.from > self.cursor {
            return;
        }

        let compl_type = if *is_const {
            CompletionType::Constant
        } else {
            CompletionType::Variable
        };

        self.symbols.insert(name.clone(), compl_type);
    }

    fn visit_block(&mut self, body: &StatementBlock<TypeEntry>) {
        if body.is_empty() {
            return;
        }
        let from = body.first().unwrap().span.from;
        let to = body.last().unwrap().span.to;

        let span = Span::new(from, to);

        if self.is_within_cursor(span) {
            for s in body {
                s.walk_visit(self);
            }
        }
    }

    fn visit_function(&mut self, name: &String, _modifier: &Modifier, _generics: &Vec<String>, _params: &Vec<Parameter>, _return_type: &TypeEntry, body: &StatementBlock<TypeEntry>, span: Span) {
        self.symbols.insert(name.clone(), CompletionType::Function);

        if self.is_within_cursor(span) {
            self.visit_block(body);
        }
    }

    fn visit_struct(&mut self, name: &String, _public_constructor: &bool, _fields: &Vec<Parameter>, body: &StatementBlock<TypeEntry>, _generics: &Vec<String>, span: Span) {
        self.symbols.insert(name.clone(), CompletionType::Struct);

        if self.is_within_cursor(span) {
            self.visit_block(body);
        }
    }

    fn visit_enum(&mut self, name: &String, _values: &Vec<String>, body: &StatementBlock<TypeEntry>, span: Span) {
        self.symbols.insert(name.clone(), CompletionType::Struct);

        if self.is_within_cursor(span) {
            self.visit_block(body);
        }
    }


}