use std::collections::HashMap;
use ordermap::map::MutableKeys;
use crate::analysis::generic_helper::GenericHelper;
use crate::analysis::mangling;
use crate::analysis::symbol_table::{SymbolTable, TypeSymTable, TypeSymTableInfo};
use crate::analysis::type_duplicator::TypeDuplicator;
use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::{ArrayType, Bool, ErroredType, Float, FunctionType, FunctionsType, GenericType, Int, NullableType, StringType, StructType, TupleType, UnknownType, Void};
use crate::ast::expression;
use crate::ast::expression::{deref, member, Expression, TypedExpr};
use crate::ast::expression::Expression::{ArrayAccessExpr, ArrayLiteralExpr, AssignExpr, BinaryExpr, BoolLiteralExpr, CallExpr, DecrementExpr, ErroredExpr, FloatLiteralExpr, IdentifierExpr, IncrementExpr, IntLiteralExpr, MemberExpr, NullDerefExpr, NullLiteralExpr, NullableExpr, PrefixExpr, StringLiteralExpr, TupleExpr};
use crate::ast::statement::{Parameter, Statement, StatementBlock, TypedStmt, UntypedStmt};
use crate::ast::statement::Statement::{FunctionStmt, ReturnStmt, StructStmt, VarDeclarationStmt};
use crate::ast::walking::folder::{FoldedExpr, FoldedStmt, Folder};
use crate::error::analysis_error::AnalysisError;
use crate::error::analysis_error::AnalysisError::{RedundantNullable, TypeResolutionFailed};
use crate::error::context::{ErrorContext, Span};
use crate::error::runtime_error::ValueTypeVariant;
use crate::lexer::token::SimpleToken;
use crate::util::spanned::Spanned;

pub struct TypeResolver<'a> {
    registry: &'a mut TypeRegistry,
    symbol_table: &'a mut TypeSymTable,
    type_table: &'a mut TypeSymTable,
    pub generic_helper: GenericHelper,
    pub errors: Vec<ErrorContext<AnalysisError>>
}


impl <'a> TypeResolver<'a> {

    pub fn new(registry: &'a mut TypeRegistry, symbol_table: &'a mut TypeSymTable, type_table: &'a mut TypeSymTable) -> Self {
        Self {
            registry, symbol_table, type_table,
            generic_helper: GenericHelper::new(),
            errors: vec![]
        }
    }

    fn error_type(&mut self, err: AnalysisError) -> AstType {
        // FIXME bad span
        self.errors.push(ErrorContext::of(err, Span::new(0, 0)));

        ErroredType
    }

    fn error(&mut self, err: AnalysisError, span: Span) -> FoldedExpr<TypeEntry> {
        self.errors.push(ErrorContext::of(err, span));

        TypedExpr::err(self.registry)
    }

    fn error_stmt(&mut self, err: AnalysisError, span: Span) -> FoldedStmt<TypeEntry> {
        self.errors.push(ErrorContext::of(err, span));

        TypedStmt::err(self.registry, span)
    }

    fn get_assign_result(&mut self, left: TypeEntry, right: TypeEntry) -> Option<AstType> {
        left.get(self.registry).get_assign_result(right.get(self.registry), self.registry)
    }

    fn deref(&self, t: TypeEntry) -> AstType {
        match t.get(self.registry) {
            NullableType { underlying } => {
                deref(underlying, self.registry)
            }

            _ => t.get(self.registry)
        }
    }

    fn get_func_key(&self, name: String, params: &Vec<Parameter>) -> String {
        let owner = self.get_current_owner(&name);
        mangling::mangle_name(self.registry, name.clone(), owner, &params)
    }

    fn get_func_key_type(&self, name: String, params: &Vec<TypeEntry>) -> String {
        let owner = self.get_current_owner(&name);
        mangling::mangle_name_type(self.registry, name.clone(), owner, &params)
    }

    fn get_current_owner(&self, name: &String) -> String {
        if let Some(data) = self.symbol_table.get_with_info(name) {
            if let Some(info) = data.1 &&
                let Some(owner_type) = info.owner &&
                let StructType{name: owner, ..} = owner_type {
                owner
            } else {
                String::new()
            }
        } else {
            panic!("Could not find {name}")
        }
    }

    fn box_arg(&mut self, arg: &mut TypedExpr, desired: TypeEntry) {
        if let AstType::GenericType {name} = desired.get(self.registry) {
            let prev = self.type_table.get(&name);

            let prev = match prev {
                Some(v) => v,
                None => {
                    panic!();
                    // let error_type = self.error_type(AnalysisError::UnknownType(name));
                    // self.registry.register(error_type)
                }
            };

            if matches!(prev.get(self.registry), UnknownType) {
                prev.mutate(self.registry, arg.get_type().get(self.registry));
            } else {
                if let Some(r) = self.get_assign_result(prev, arg.get_type()) {
                    prev.mutate(self.registry, r);
                } else {
                    self.error_type(AnalysisError::illegal_type_assignment(prev, arg.get_type(), self.registry));
                }
            }


            // type_table.record(name, arg.get_type());
            desired.mutate(self.registry, arg.get_type().get(self.registry));
        }

        // arg does not know its type
        if matches!(arg.get_type().get(self.registry), UnknownType) {
            arg.set_type(self.registry, desired.get(self.registry));
            return;
        }

        // arg is '(<unknown>)?'
        if let NullableType {underlying} = arg.get_type().get(self.registry) && matches!(underlying.get(self.registry), UnknownType) {
            if !matches!(desired.get(self.registry), NullableType {..}) {
                panic!("AAAA WTF");
            }

            arg.set_type(self.registry, desired.get(self.registry));
            return;
        }

        if matches!(desired.get(self.registry), NullableType {..}) && !matches!(arg.get_type().get(self.registry), NullableType {..}) {
            *arg = Spanned::of(NullableExpr(self.registry.register(NullableType { underlying: arg.get_type() }), arg.clone().boxed()), arg.span);
        }
    }


}

impl<'a> Folder<(), TypeEntry> for TypeResolver<'a> {
    fn get_registry(&self) -> &TypeRegistry {
        self.registry
    }

    fn get_registry_mut(&mut self) -> &mut TypeRegistry {
        self.registry
    }

    fn fold_annotation(&mut self, t: ()) -> TypeEntry {
        panic!("Should not be called");
    }

    fn fold_type_entry(&mut self, t: TypeEntry) -> TypeEntry {
        let ast_type = t.get(self.registry);

        let result = ast_type.walk_fold(self);

        t.mutate(self.registry, result);

        t
    }

    fn fold_var_declaration(&mut self, name: String, is_const: bool, value: Spanned<Expression<()>>, explicit_type: Option<TypeEntry>, span: Span) -> FoldedStmt<TypeEntry> {
        let mut value = self.fold_expr(value);

        // still record that *some* variable exists even if it's an error
        self.symbol_table.record(name.clone(), value.get_type());

        if value.is_err(self.registry) {
            return TypedStmt::err(self.registry, span);
        }

        let mut resolved_expl_type = None;

        if let Some(t) = explicit_type {
            let t = self.fold_type_entry(t);

            if t.is_err(self.registry) {
                return TypedStmt::err(self.registry, span);
            }

            match self.get_assign_result(t, value.get_type()) {
                Some(new_t) => value.set_type(self.registry, new_t),
                None => return self.error_stmt(AnalysisError::illegal_type_assignment(t, value.get_type(), self.registry), span)
            }

            resolved_expl_type = Some(t);
        }

        VarDeclarationStmt {name, is_const, value, explicit_type: resolved_expl_type}
    }

    fn fold_function(&mut self, name: String, generics: Vec<String>, params: Vec<Parameter>, return_type: TypeEntry, body: StatementBlock<()>, span: Span) -> FoldedStmt<TypeEntry> {
        self.type_table.push();
        for g in generics.iter() {
            self.type_table.record(g.clone(), self.registry.register(GenericType {name: g.clone()}));
        }

        let return_type = self.fold_type_entry(return_type);

        let mut typed_params = vec![];

        for p in params.iter() {
            typed_params.push(Parameter{
                name: p.name.clone(),
                param_type: self.fold_type_entry(p.param_type)
            });
        }

        self.symbol_table.push();

        self.symbol_table.record_return_type(return_type);

        let mut has_generics_params = matches!(return_type.get(self.registry), GenericType {..});
        for p in typed_params.clone().iter() {
            if matches!(p.param_type.get(self.registry), GenericType {..}) {
                has_generics_params = true;
            }

            self.symbol_table.record(p.name.clone(), p.param_type);
        }

        let typed_body = self.fold_block(body.clone());

        self.symbol_table.pop();
        self.type_table.pop();

        if !generics.is_empty() || has_generics_params {
            let key = self.get_func_key(name.clone(), &params);

            self.generic_helper.record_generic(key, name.clone(), params, return_type, body, span);
        }

        FunctionStmt {name, generics, params: typed_params, return_type, body: typed_body}
    }

    fn fold_struct(&mut self, name: String, fields: Vec<Parameter>, body: StatementBlock<()>, generics: Vec<String>, span: Span) -> FoldedStmt<TypeEntry> {
        let mut typed_fields = vec![];

        let struct_type = self.symbol_table.get(&name).unwrap();

        self.type_table.push();
        if let StructType {generics,..} = struct_type.get(self.registry) {
            for e in generics {
                self.type_table.record(e.0.clone(), self.registry.register(GenericType {name: e.0}));
            }
        } else {
            return self.error_stmt(AnalysisError::type_mismatch(ValueTypeVariant::Struct, struct_type, self.registry), span);
        }

        for p in fields {
            typed_fields.push(Parameter{
                name: p.name,
                param_type: self.fold_type_entry(p.param_type)
            });
        }

        self.symbol_table.push();

        self.symbol_table.record(String::from("this"), struct_type.clone());


        if let StructType {children,..} = struct_type.get(self.registry) {
            for p in children {
                self.symbol_table.record_with_info(p.0, p.1.0, TypeSymTableInfo::inside_struct(struct_type.get(self.registry)));
            }
        } else {
            return self.error_stmt(AnalysisError::type_mismatch(ValueTypeVariant::Struct, struct_type, self.registry), span);
        }

        let body = self.fold_block(body);

        self.symbol_table.pop();
        self.type_table.pop();

        StructStmt {name, fields: typed_fields, body, generics}
    }

    fn fold_return(&mut self, expr: Spanned<Expression<()>>, span: Span) -> FoldedStmt<TypeEntry> {
        let mut expr = self.fold_expr(expr);

        let return_type = self.symbol_table.get_return_type();

        let return_type = match return_type {
            Some(r) => r,
            None => return self.error_stmt(AnalysisError::DanglingReturn, span)
        };

        self.box_arg(&mut expr, return_type);

        ReturnStmt(expr)
    }


    fn fold_errored(&mut self, t: (), span: Span) -> FoldedExpr<TypeEntry> {
        ErroredExpr(self.registry.register(ErroredType))
    }

    fn fold_null_literal(&mut self, t: (), span: Span) -> FoldedExpr<TypeEntry> {
        let unknown = self.registry.register(UnknownType);

        let t = NullableType {underlying: unknown};

        NullLiteralExpr(self.registry.register(t))
    }

    fn fold_bool_literal(&mut self, t: (), value: bool, span: Span) -> FoldedExpr<TypeEntry> {
        BoolLiteralExpr(self.registry.register(Bool), value)
    }

    fn fold_int_literal(&mut self, t: (), value: i64, span: Span) -> FoldedExpr<TypeEntry> {
        IntLiteralExpr(self.registry.register(Int), value)
    }

    fn fold_float_literal(&mut self, t: (), value: f32, span: Span) -> FoldedExpr<TypeEntry> {
        FloatLiteralExpr(self.registry.register(Float), value)
    }

    fn fold_string_literal(&mut self, t: (), value: String, span: Span) -> FoldedExpr<TypeEntry> {
        StringLiteralExpr(self.registry.register(StringType), value)
    }

    fn fold_array_literal(&mut self, t: (), values: Vec<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        if values.is_empty() {
            let typ = ArrayType {underlying: self.registry.register(UnknownType)};

            return ArrayLiteralExpr(self.registry.register(typ), Vec::with_capacity(0))
        }

        let mut typed_values = vec![];

        for v in values {
            typed_values.push(self.fold_expr(v));
        }

        let mut underlying = typed_values[0].get_type();

        for v in &typed_values {
            if let Some(res) = underlying.get(self.registry).get_assign_result(v.get_type().get(self.registry), self.registry) {
                underlying = self.registry.register(res);
            } else if let Some(res) = v.get_type().get(self.registry).get_assign_result(underlying.get(self.registry), self.registry) {
                underlying = self.registry.register(res);
            }
        }

        for v in typed_values.iter_mut() {
            v.set_type(self.registry, underlying.get(self.registry))
        }

        let array_type = ArrayType { underlying };

        ArrayLiteralExpr(self.registry.register(array_type), typed_values)
    }

    fn fold_identifier(&mut self, t: (), name: String, span: Span) -> FoldedExpr<TypeEntry> {
        let v = self.symbol_table.get_with_info(&name);

        if let Some(tuple) = v {
            let t = tuple.0;
            let info = tuple.1;

            if let Some(v) = info && v.inside_struct {

                let this = Spanned::new(IdentifierExpr((), "this".to_string()), span.from, span.from);
                let this = self.fold_expr(this);

                MemberExpr {
                    t: t.clone(),
                    member: this.boxed(),
                    property: Spanned::of(IdentifierExpr(t, name), span).boxed(),
                    null_safe: false
                }
            } else {
                IdentifierExpr(t, name)
            }
        } else {
            self.error(AnalysisError::UnknownType(name), span)
        }
    }

    fn fold_increment(&mut self, t: (), expr: Box<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let typed = self.fold_expr(*expr);

        IncrementExpr(typed.get_type(), typed.boxed())
    }

    fn fold_decrement(&mut self, t: (), expr: Box<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let typed = self.fold_expr(*expr);

        DecrementExpr(typed.get_type(), typed.boxed())
    }

    fn fold_null_deref(&mut self, t: (), expr: Box<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let typed = self.fold_expr(*expr);

        if let NullableType {underlying} = typed.get_type().get(self.registry) {
            NullDerefExpr(underlying, typed.boxed())
        } else {
            self.error(AnalysisError::illegal_null_deref(typed.get_type(), self.registry), span)
        }
    }

    fn fold_prefix(&mut self, t: (), operator: SimpleToken, expr: Box<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let typed = self.fold_expr(*expr);

        PrefixExpr {t: typed.get_type(), operator, expr: typed.boxed()}
    }

    fn fold_binary(&mut self, t: (), left: Box<Spanned<Expression<()>>>, operator: SimpleToken, right: Box<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let left = self.fold_expr(*left);
        let right = self.fold_expr(*right);

        if left.is_err(self.registry) || right.is_err(self.registry) {
            return TypedExpr::err(self.registry);
        }

        let result_type = self.symbol_table.op_table.get_op_result(self.registry, left.get_type(), operator, right.get_type());

        let result_type = match result_type {
            Ok(r) => r,
            Err(e) => return self.error(e, span)
        };

        let t = self.registry.register(result_type);

        BinaryExpr {t, left: left.boxed(), operator, right: right.boxed()}
    }

    fn fold_assign(&mut self, t: (), assignee: Box<Spanned<Expression<()>>>, value: Box<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let mut assignee = self.fold_expr(*assignee);
        let mut value = self.fold_expr(*value);

        if assignee.is_err(self.registry) || value.is_err(self.registry) {
            return TypedExpr::err(self.registry);
        }

        let assign_result = self.get_assign_result(assignee.get_type(), value.get_type());

        if let Some(t) = assign_result {
            assignee.set_type(self.registry, t.clone());

            if matches!(assignee.get_type().get(self.registry), NullableType {..}) && !matches!(value.get_type().get(self.registry), NullableType {..}) {
                let nullable = self.registry.register(NullableType {underlying: value.get_type()});

                let expr = NullableExpr(nullable, value.clone().boxed());
                value = Spanned::of(expr, value.span);
            }
        } else {
            return self.error(AnalysisError::illegal_type_assignment(assignee.get_type(), value.get_type(), self.registry), span);
        }

        let t = assignee.get_type();

        AssignExpr {t, assignee: assignee.boxed(), value: value.boxed()}
    }

    fn fold_array_access(&mut self, t: (), property: Box<Spanned<Expression<()>>>, index: Box<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let property = self.fold_expr(*property);
        let index = self.fold_expr(*index);

        if property.is_err(self.registry) || index.is_err(self.registry) {
            return TypedExpr::err(self.registry);
        }

        let underlying;
        if let ArrayType {underlying: u} = property.get_type().get(self.registry) {
            underlying = u;
        } else {
            return self.error(AnalysisError::illegal_indexing(property.get_type(), self.registry), span);
        }


        ArrayAccessExpr {
            t: underlying,
            property: property.boxed(),
            index: index.boxed()
        }
    }

    fn fold_tuple(&mut self, t: (), values: Vec<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let mut types = vec![];

        for v in values {
            types.push(self.fold_expr(v));
        }

        let t = TupleType(types.iter().map(|e| e.get_type()).collect());

        let tuple_type = self.registry.register(t);

        TupleExpr {t: tuple_type, values: types}
    }

    fn fold_member(&mut self, t: (), member: Box<Spanned<Expression<()>>>, property: Box<Spanned<Expression<()>>>, null_safe: bool, span: Span) -> FoldedExpr<TypeEntry> {
        let member = self.fold_expr(*member);

        // if we are accessing something on a struct, temporarily add the structs methods and fields into scope
        let mut struct_scope = false;
        if let AstType::StructType{name, children, generics, ..} = self.deref(member.get_type()) {
            struct_scope = true;
            self.symbol_table.push();
            for x in children {
                self.symbol_table.record_with_info(x.0, x.1.0, TypeSymTableInfo::owner(self.deref(member.get_type())));
            }

            self.type_table.push();
            for g in generics {
                self.type_table.record(g.0, g.1);
            }
        }


        let property = self.fold_expr(*property);
        let mut typ = property.get_type();

        if null_safe {
            let nullable = NullableType {underlying: typ};
            typ = self.registry.register(nullable)
        }

        // drop the struct scope
        if struct_scope {
            self.type_table.pop();
            self.symbol_table.pop();
        }

        MemberExpr {
            t: typ,
            member: member.boxed(),
            property: property.boxed(),
            null_safe
        }
    }

    // FIXME refactor this monster
    fn fold_call(&mut self, t: (), func: Box<Spanned<Expression<()>>>, args: Vec<Spanned<Expression<()>>>, span: Span) -> FoldedExpr<TypeEntry> {
        let resolved_func = self.fold_expr(*func);
        let mut resolved_func = TypeDuplicator::duplicate(self.registry, resolved_func);

        if let FunctionsType { overloads, name, .. } = resolved_func.get_type().get(self.registry) {
            let mut resolved_args = vec![];

            let mut err = false;
            for arg in args.clone() {
                let arg = self.fold_expr(arg);

                if arg.is_err(self.registry) {
                    err = true;
                }
                resolved_args.push(arg);
            }

            if err {
                return TypedExpr::err(self.registry);
            }

            // FIXME leaking the void here a bit
            let mut func = self.registry.register(Void);

            'overloadLoop:
            for overload in overloads.iter() {
                if let AstType::FunctionType { name, generics, params, .. } = overload.get(self.registry) {
                    if params.len() != resolved_args.len() {
                        continue;
                    }

                    for i in 0..resolved_args.len() {
                        if !params[i].get(self.registry).loose_equals(&resolved_args[i].get_type().get(self.registry), self.registry) {
                            continue 'overloadLoop;
                        }
                    }

                    func = *overload;
                    break;
                } else {
                    panic!();
                }
            }


            if let AstType::FunctionType { name, generics, mut return_type, params, .. } = func.get(self.registry) {
                let key = self.get_func_key_type(name.clone(), &params);

                self.type_table.push();

                for e in generics.clone() {
                    self.type_table.record(e.0.clone(), e.1);
                }

                // resolve argument types
                for i in 0..params.len() {
                    let arg = &mut resolved_args[i];
                    let p = params[i];

                    // auto null-boxing
                    self.box_arg(arg, p);
                }

                // FIXME not at all sure if `set_type` or `change_type` should be called here aaaa
                resolved_func.change_type(self.registry, func.get(self.registry));

                self.type_table.pop();


                self.type_table.push();
                if let FunctionType {params,generics,..} = resolved_func.get_type().get(self.registry) {
                    for g in generics {
                        self.type_table.record(g.0, g.1);
                    }
                }

                if let Some(original) = self.generic_helper.get_generic(self.registry, &key) {
                    let mut struct_scope = false;

                    if let Some(record) =self.symbol_table.get_with_info(&name) &&
                        let Some(info) = record.1 &&
                        let Some(struct_type) = info.owner {
                        struct_scope = true;

                        let registered = self.registry.register(struct_type.clone());

                        self.symbol_table.push();
                        self.symbol_table.record(String::from("this"), registered);

                        if let StructType {children,..} = &struct_type {
                            for p in children {
                                self.symbol_table.record_with_info(p.0.clone(), p.1.0, TypeSymTableInfo::inside_struct(struct_type.clone()));
                            }
                        } else {
                            return self.error(AnalysisError::type_mismatch(ValueTypeVariant::Struct, registered, self.registry), span);
                        }
                    }


                    let resolved = self.fold_stmt(original);
                    self.type_table.pop();

                    if let FunctionStmt { name, params, return_type: resolved_return, .. } = &resolved.node {
                        return_type = *resolved_return;
                    }
                    self.generic_helper.record_implementation(key, resolved);

                    if struct_scope {
                        self.symbol_table.pop();
                    }
                }

                if let MemberExpr { t, member, property, null_safe } = &resolved_func.node
                    && let IdentifierExpr(t, name) = &member.node && name == "this"
                {
                    let mut property = property.clone();
                    property.change_type(self.registry, func.get(self.registry));


                    return MemberExpr {
                        t: return_type,
                        member: member.clone().boxed(),
                        null_safe: *null_safe,
                        property: Spanned::of(CallExpr {
                            t: return_type,
                            func: property.clone(),
                            args: resolved_args

                            // FIXME I think the span should be combined with args here
                        }, property.span).boxed()
                    };
                }

                if resolved_func.is_err(self.registry) {
                    return TypedExpr::err(self.registry);
                }

                return CallExpr {
                    t: return_type,
                    func: resolved_func.boxed(),
                    args: resolved_args
                };
            }
        }

        // calling constructor of a struct
        if let AstType::StructType { fields,generics, mut children, .. } = resolved_func.get_type().get(self.registry) {
            let mut resolved_args = vec![];

            let mut err = false;
            for arg in args.clone() {
                let arg = self.fold_expr(arg);

                if arg.is_err(self.registry) {
                    err = true;
                }
                resolved_args.push(arg);
            }

            if err {
                return TypedExpr::err(self.registry);
            }

            if fields.len() != resolved_args.len() {
                panic!("Invalid constructor call");
            }

            self.type_table.push();

            for e in generics.clone() {
                self.type_table.record(e.0.clone(), e.1);
            }

            for i in 0..fields.len() {
                let field = &fields[i].0;
                let arg = &mut resolved_args[i];

                self.box_arg(arg, *field);
            }

            if resolved_func.is_err(self.registry) {
                return TypedExpr::err(self.registry);
            }

            self.type_table.pop();

            return CallExpr {
                t: resolved_func.get_type(),
                func: resolved_func.boxed(),
                args: resolved_args
            };
        }

        if resolved_func.is_err(self.registry) {
            return TypedExpr::err(self.registry);
        }

        self.error(AnalysisError::illegal_call(resolved_func.get_type(), self.registry), span)
    }

    fn fold_symbol_type(&mut self, name: String, generics: Vec<TypeEntry>) -> AstType {
        let opt = self.type_table.get(&name);

        if let Some(t) = opt {
            if !generics.is_empty() {
                let mut res = t.get(self.registry);

                if let StructType {generics: g, ..} = &mut res {
                    if generics.len() != g.len() {
                        panic!();
                    }

                    let mut iter = generics.iter();

                    for x in g.iter_mut() {
                        *x.1 = *iter.next().unwrap();
                    }
                }

                return res;
            }

            return t.get(self.registry);
        }

        self.error_type(TypeResolutionFailed(name))
    }

    fn fold_nullable_type(&mut self, underlying: TypeEntry) -> AstType {
        self.fold_type_entry(underlying);

        if matches!(underlying.get(self.registry), NullableType {..}) {
            return self.error_type(RedundantNullable);
        }

        NullableType {underlying}
    }

    fn fold_generic_type(&mut self, name: String) -> AstType {
        self.type_table.get(&name).unwrap().get(self.registry)
    }


}
