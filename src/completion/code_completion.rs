use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::expression::{deref, TypedExpr};
use crate::ast::modifier::Modifier;
use crate::ast::statement::{Parameter, StatementBlock, TypedStmt};
use crate::ast::walking::visitor::Visitor;
use crate::completion::completion_type::CompletionType;
use crate::error::context::Span;
use crate::intrinsics::type_injector;
use crate::lexer::token::SimpleToken;
use std::collections::{HashMap, HashSet};

pub fn get_code_suggestions(cursor: usize, code_text: &str, code: &TypedStmt, registry: &mut TypeRegistry) -> HashMap<String, CompletionType> {
    if let Some(dot) = find_member_dot(code_text, cursor) {
        let receiver_name = identifier_before(code_text, dot);

        if let Some(members) = member_suggestions(dot, receiver_name, code, registry) {
            return members;
        }
    }

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
            suggestions.insert(name, CompletionType::Struct);
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


fn find_member_dot(text: &str, cursor: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut i = cursor.min(bytes.len());

    while i > 0 && is_ident_byte(bytes[i - 1]) {
        i -= 1;
    }
    while i > 0 && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }

    if i > 0 && bytes[i - 1] == b'.' {
        // don't treat `..` as member access
        if i >= 2 && bytes[i - 2] == b'.' {
            return None;
        }
        return Some(i - 1);
    }

    None
}

fn is_ident_byte(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}


fn identifier_before(text: &str, dot: usize) -> Option<String> {
    let bytes = text.as_bytes();

    let end = dot;
    let mut start = end;

    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    if start == end {
        return None;
    }
    Some(text[start..end].to_string())
}

fn member_suggestions(dot: usize, receiver_name: Option<String>, code: &TypedStmt, registry: &TypeRegistry) -> Option<HashMap<String, CompletionType>> {
    let allow_private = receiver_name.as_deref() == Some("this");
    let receiver_type = resolve_receiver_type(dot, receiver_name, code, registry)?;

    let mut out = HashMap::new();

    match deref(receiver_type, registry) {
        AstType::StructType { fields, children, .. } => {
            let field_names: HashSet<&String> = fields.iter().map(|f| &f.name).collect();

            for f in &fields {
                if f.public || allow_private {
                    out.insert(f.name.clone(), CompletionType::Field);
                }
            }
            for (name, info) in &children {
                if field_names.contains(name) {
                    continue;
                }
                if allow_private || method_is_public(info.typ, registry) {
                    out.insert(name.clone(), CompletionType::Method);
                }
            }
        }
        _ => return None,
    }

    if out.is_empty() {
        return None;
    }

    Some(out)
}

fn method_is_public(typ: TypeEntry, registry: &TypeRegistry) -> bool {
    match typ.get(registry) {
        AstType::FunctionsType { overloads, .. } => overloads.iter().any(|o| {
            matches!(o.get(registry), AstType::FunctionType { modifier, .. } if modifier.is_public())
        }),
        AstType::FunctionType { modifier, .. } => modifier.is_public(),
        _ => false,
    }
}


fn resolve_receiver_type(dot: usize, receiver_name: Option<String>, code: &TypedStmt, registry: &TypeRegistry) -> Option<TypeEntry> {
    let mut finder = ReceiverFinder { registry, dot, best: None };
    code.walk_visit(&mut finder);

    if let Some((_, t)) = finder.best {
        if !t.is_err(registry) {
            return Some(t);
        }
    }

    let name = receiver_name?;
    let mut name_finder = NameTypeFinder { registry, target: name, dot, found: None };

    code.walk_visit(&mut name_finder);
    name_finder.found.map(|(_, t)| t)
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


struct ReceiverFinder<'a> {
    registry: &'a TypeRegistry,
    dot: usize,
    best: Option<(usize, TypeEntry)>,
}

impl<'a> Visitor<'a> for ReceiverFinder<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn visit_type(&mut self, _typ: &TypeEntry) {}

    fn visit_expr(&mut self, expr: &TypedExpr) {
        if expr.span.to == self.dot {
            let from = expr.span.from;
            let better = match self.best {
                Some((best_from, _)) => from < best_from,
                None => true,
            };
            if better {
                self.best = Some((from, expr.get_type()));
            }
        }

        expr.walk_visit(self);
    }
}


struct NameTypeFinder<'a> {
    registry: &'a TypeRegistry,
    target: String,
    dot: usize,
    found: Option<(usize, TypeEntry)>,
}

impl<'a> NameTypeFinder<'a> {
    fn consider(&mut self, at: usize, name: &str, typ: TypeEntry) {
        if at > self.dot || name != self.target {
            return;
        }
        let better = match self.found {
            Some((best_at, _)) => at >= best_at,
            None => true,
        };
        if better {
            self.found = Some((at, typ));
        }
    }
}

impl<'a> Visitor<'a> for NameTypeFinder<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn visit_type(&mut self, _typ: &TypeEntry) {}

    fn visit_var_declaration(&mut self, name: &String, _is_const: &bool, value: &TypedExpr, _explicit_type: &Option<TypeEntry>, span: Span) {
        self.consider(span.from, name, value.get_type());
    }

    fn visit_function(&mut self, _name: &String, _modifier: &Modifier, _generics: &Vec<String>, params: &Vec<Parameter>, _return_type: &TypeEntry, body: &StatementBlock<TypeEntry>, span: Span) {
        if is_within_cursor(self.dot, span) {
            for p in params {
                self.consider(span.from, &p.name, p.param_type);
            }
        }
        for s in body {
            s.walk_visit(self);
        }
    }
}


struct SymbolCollector<'a> {
    registry: &'a TypeRegistry,
    symbols: HashMap<String, CompletionType>,
    cursor: usize,
    inside_struct: bool,
}

impl <'a> SymbolCollector<'a> {

    pub fn new(registry: &'a TypeRegistry, cursor: usize) -> Self {
        Self {
            registry, cursor,
            symbols: HashMap::new(),
            inside_struct: false,
        }
    }

    fn is_within_cursor(&self, span: Span) -> bool {
        span.from <= self.cursor && span.to > self.cursor
    }

}

fn is_within_cursor(cursor: usize, span: Span) -> bool {
    span.from <= cursor && span.to > cursor
}

impl <'a> Visitor<'a> for SymbolCollector<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn visit_type(&mut self, typ: &TypeEntry) {
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

    fn visit_function(&mut self, name: &String, _modifier: &Modifier, _generics: &Vec<String>, params: &Vec<Parameter>, _return_type: &TypeEntry, body: &StatementBlock<TypeEntry>, span: Span) {
        let kind = if self.inside_struct {
            CompletionType::Method
        } else {
            CompletionType::Function
        };
        self.symbols.insert(name.clone(), kind);

        if self.is_within_cursor(span) {
            for p in params {
                self.symbols.insert(p.name.clone(), CompletionType::Parameter);
            }
            self.visit_block(body);
        }
    }

    fn visit_struct(&mut self, name: &String, _public_constructor: &bool, fields: &Vec<Parameter>, body: &StatementBlock<TypeEntry>, _generics: &Vec<String>, span: Span) {
        self.symbols.insert(name.clone(), CompletionType::Struct);

        if self.is_within_cursor(span) {
            for f in fields {
                self.symbols.insert(f.name.clone(), CompletionType::Field);
            }
            self.inside_struct = true;
            self.visit_block(body);
            self.inside_struct = false;
        }
    }

    fn visit_enum(&mut self, name: &String, values: &Vec<String>, body: &StatementBlock<TypeEntry>, span: Span) {
        self.symbols.insert(name.clone(), CompletionType::Enum);

        if self.is_within_cursor(span) {
            for v in values {
                self.symbols.insert(v.clone(), CompletionType::EnumValue);
            }
            self.visit_block(body);
        }
    }


}
