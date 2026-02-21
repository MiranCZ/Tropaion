use crate::ast::ast_type::AstType;
use crate::ast::expression::Expression;
use crate::ast::statement::TypedStmt;
use crate::compiler::codegen::BytecodeGen;
use crate::compiler::expr_gen::Operation::Load;

impl TypedStmt {

    pub fn gen_bytecode(&self, generator: &mut BytecodeGen) {
        match self {
            TypedStmt::BlockStmt { body } => {
                generator.new_scope();

                for b in body {
                    b.gen_bytecode(generator);
                }

                generator.end_scope();
            }
            TypedStmt::ExpressionStmt(e) => {
                e.generate_bytecode(generator, Load);
            }
            TypedStmt::VarDeclarationStmt {name, value, .. } => {
                value.generate_bytecode(generator, Load);

                match value.get_type() {
                    AstType::Bool | AstType::Int => generator.i_store(name.clone()),
                    AstType::Float => generator.f_store(name.clone()),
                    _ => panic!("Not yet supported type {:?}",value.get_type())
                };
            }
            TypedStmt::IfStmt { condition, body, else_branch } => {
                condition.generate_bytecode(generator, Load);
                generator.new_skippable_scope();

                for b in body {
                    b.gen_bytecode(generator);
                }

                if let Some(br) = else_branch {
                    generator.push_scope_exit_insn();

                    br.gen_bytecode(generator);
                }

                generator.end_scope();
            }
            TypedStmt::WhileStmt { condition, body } => {
                condition.generate_bytecode(generator, Load);

                generator.new_skippable_scope();

                for b in body {
                    b.gen_bytecode(generator);
                }

                // FIXME should not generate this twice
                condition.generate_bytecode(generator, Load);
                generator.push_goto_scope_start_insn();

                generator.end_scope();
            }
            TypedStmt::FunctionStmt {name, body, return_type, .. } => {
                generator.comment(format!("fn {name} -- START"));
                generator.fn_start();

                for b in body {
                    b.gen_bytecode(generator);
                }

                // FIXME technically its fine adding two return statements, but not ideal
                // (this is here if no explicit return statement was inserted)
                generator.ret(0);

                generator.fn_end();


                generator.comment(format!("return of {name} -- END"));
            }
            TypedStmt::StructStmt { .. } => {}
            TypedStmt::ReturnStmt(e) => {
                 e.generate_bytecode(generator, Load);

                generator.ret(e.get_type().word_size());
            }

            // ignored (at least for now)
            TypedStmt::CommentStmt(_) => {}
            TypedStmt::MultilineCommentStmt(_) => {}
        }

    }

}