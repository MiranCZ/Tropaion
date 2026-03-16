use crate::analysis::type_registry::TypeRegistry;
use crate::ast::statement::{Statement, TypedStmt};
use crate::compiler::codegen::BytecodeGen;
use crate::compiler::expr_gen::Operation::Load;
use crate::error::compilation_error::EmptyRes;
use crate::error::ok;

impl TypedStmt {

    pub fn gen_bytecode(&self,registry: &TypeRegistry ,generator: &mut BytecodeGen) -> EmptyRes {
        generator.push_span(self.span);

        self._generate_bytecode(registry, generator)?;

        generator.pop_span();

        ok()
    }

    fn _generate_bytecode(&self, registry: &TypeRegistry, generator: &mut BytecodeGen) -> EmptyRes {
        match &self.node {
            Statement::BlockStmt { body } => {
                generator.new_scope();

                for b in body {
                    b.gen_bytecode(registry, generator)?;
                }

                generator.end_scope()?;
            }
            Statement::ExpressionStmt(e) => {
                e.generate_bytecode(registry, generator, Load)?;
            }
            Statement::VarDeclarationStmt { name, value, .. } => {
                value.generate_bytecode(registry, generator, Load)?;

                generator.store_new_var(name.clone(), registry, value.get_type())?;
            }
            Statement::IfStmt { condition, body, else_branch } => {
                condition.generate_bytecode(registry, generator, Load)?;
                generator.new_skippable_scope_eq();

                for b in body {
                    b.gen_bytecode(registry, generator)?;
                }

                if let Some(br) = else_branch {
                    // generator.push_scope_exit_insn();
                    generator.nop();
                    generator.end_scope()?;
                    generator.pop_insn();

                    generator.new_scope();
                    generator.push_scope_exit_insn();

                    br.gen_bytecode(registry, generator)?;
                    generator.end_scope()?;
                } else {
                    generator.end_scope()?;
                }
            }
            Statement::WhileStmt { condition, body } => {
                condition.generate_bytecode(registry, generator, Load)?;

                generator.new_skippable_scope_eq();

                for b in body {
                    b.gen_bytecode(registry, generator)?;
                }

                // FIXME should not generate this twice
                condition.generate_bytecode(registry, generator, Load)?;
                generator.push_goto_scope_start_insn();

                generator.end_scope()?;
            }
            Statement::FunctionStmt { name, body, params, .. } => {
                generator.comment(format!("fn {name} -- START"));
                generator.fn_start(name.clone());

                for param in params.iter().rev() {
                    let name = param.name.clone();

                    generator.store_new_var(name, registry, param.param_type)?;
                }

                for b in body {
                    b.gen_bytecode(registry, generator)?;
                }

                // FIXME technically its fine adding two return statements, but not ideal
                // (this is here if no explicit return statement was inserted)
                generator.ret(0);

                generator.fn_end(name.clone(), registry)?;


                generator.comment(format!("return of {name} -- END"));
            }
            Statement::StructStmt { body, .. } => {
                for b in body {
                    b.gen_bytecode(registry, generator)?;
                }
            }
            Statement::ReturnStmt(e) => {
                e.generate_bytecode(registry, generator, Load)?;

                generator.ret(e.get_type().get(registry).word_size(registry));
            }
            
            Statement::LoopInterrupt {break_loop} => {
                todo!()
            }

            // ignored (at least for now)
            Statement::CommentStmt(_) => {}
            Statement::MultilineCommentStmt(_) => {}
        }
        Ok(())
    }
}