use crate::analysis::type_registry::{TypeEntry, TypeRegistry};
use crate::ast::ast_type::AstType;
use crate::ast::expression::Expression;
use crate::ast::statement::TypedStmt;
use crate::compiler::codegen::BytecodeGen;
use crate::compiler::expr_gen::Operation::Load;

impl TypedStmt {

    pub fn gen_bytecode(&self,registry: &TypeRegistry ,generator: &mut BytecodeGen) {
        match self {
            TypedStmt::BlockStmt { body } => {
                generator.new_scope();

                for b in body {
                    b.gen_bytecode(registry, generator);
                }

                generator.end_scope();
            }
            TypedStmt::ExpressionStmt(e) => {
                e.generate_bytecode(registry, generator, Load);
            }
            TypedStmt::VarDeclarationStmt {name, value, .. } => {
                value.generate_bytecode(registry, generator, Load);

                generator.store_new_var(name.clone(), registry, value.get_type());
            }
            TypedStmt::IfStmt { condition, body, else_branch } => {
                condition.generate_bytecode(registry, generator, Load);
                generator.new_skippable_scope_eq();

                for b in body {
                    b.gen_bytecode(registry, generator);
                }

                if let Some(br) = else_branch {
                    // generator.push_scope_exit_insn();
                    generator.nop();
                    generator.end_scope();
                    generator.instructions.pop();

                    generator.new_scope();
                    generator.push_scope_exit_insn();

                    br.gen_bytecode(registry, generator);
                    generator.end_scope();
                } else {
                    generator.end_scope();
                }

            }
            TypedStmt::WhileStmt { condition, body } => {
                condition.generate_bytecode(registry, generator, Load);

                generator.new_skippable_scope_eq();

                for b in body {
                    b.gen_bytecode(registry, generator);
                }

                // FIXME should not generate this twice
                condition.generate_bytecode(registry, generator, Load);
                generator.push_goto_scope_start_insn();

                generator.end_scope();
            }
            TypedStmt::FunctionStmt {name, body, params, return_type, .. } => {
                generator.comment(format!("fn {name} -- START"));
                generator.fn_start(name.clone());

                for param in params {
                    let name = param.name.clone();

                    generator.store_new_var(name, registry, param.param_type);
                }

                for b in body {
                    b.gen_bytecode(registry, generator);
                }

                // FIXME technically its fine adding two return statements, but not ideal
                // (this is here if no explicit return statement was inserted)
                generator.ret(0);

                generator.fn_end(name.clone());


                generator.comment(format!("return of {name} -- END"));
            }
            TypedStmt::StructStmt {name, fields, body, .. } => {

                for b in body {
                    b.gen_bytecode(registry, generator);
                }
            }
            TypedStmt::ReturnStmt(e) => {
                 e.generate_bytecode(registry, generator, Load);

                generator.ret(e.get_type().get(registry).word_size(registry));
            }

            // ignored (at least for now)
            TypedStmt::CommentStmt(_) => {}
            TypedStmt::MultilineCommentStmt(_) => {}
        }

    }

}