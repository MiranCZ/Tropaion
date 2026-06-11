use crate::analysis::type_registry::TypeRegistry;
use crate::ast::statement::{Statement, TypedStmt};
use crate::compiler::codegen::BytecodeGen;
use crate::compiler::expr_gen::Operation::Load;
use crate::error::compilation_error::{CompilationError, EmptyRes};
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
                    generator.push_scope_exit_insn()?;

                    br.gen_bytecode(registry, generator)?;
                    generator.end_scope()?;
                } else {
                    generator.end_scope()?;
                }
            }
            Statement::WhileStmt { condition, body } => {
                generator.loop_label();
                condition.generate_bytecode(registry, generator, Load)?;

                generator.new_skippable_loop_eq();

                for b in body {
                    b.gen_bytecode(registry, generator)?;
                }

                // FIXME should not generate this twice
                // condition.generate_bytecode(registry, generator, Load)?;
                generator.push_goto_loop_start_insn();

                generator.end_scope()?;
            }
            Statement::FunctionStmt { name, body, params, .. } => {
                generator.fn_start(name.clone())?;

                for param in params.iter().rev() {
                    let name = param.name.clone();

                    generator.store_param(name, registry, param.param_type)?;
                }

                for b in body {
                    b.gen_bytecode(registry, generator)?;
                }

                // FIXME technically its fine adding two return statements, but not ideal
                // (this is here if no explicit return statement was inserted)
                generator.ret(0);

                generator.fn_end(name.clone(), registry)?;
            }
            Statement::ConstructorStmt {..} => {
                return Err(CompilationError::InternalError("Constructors are syntactic sugar!".to_string()));
            }
            Statement::StructStmt { body, .. } => {
                for b in body {
                    b.gen_bytecode(registry, generator)?;
                }
            }
            Statement::EnumStmt {body, ..} => {
                for b in body {
                    b.gen_bytecode(registry, generator)?;
                }
            }
            Statement::ReturnStmt(expr) => {
                if let Some(e) = expr {
                    e.generate_bytecode(registry, generator, Load)?;

                    generator.ret(e.get_type().get(registry).word_size(registry));
                } else {
                    generator.ret(0)
                }
            }

            Statement::LoopInterrupt {break_loop} => {
                if *break_loop {
                    generator.push_loop_exit_insn();
                } else {
                    generator.push_goto_loop_start_insn();
                }
            }

            // ignored (at least for now)
            Statement::CommentStmt(_) => {}
            Statement::MultilineCommentStmt(_) => {}
        }
        Ok(())
    }
}