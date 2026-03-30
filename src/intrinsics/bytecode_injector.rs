use crate::analysis::type_registry::TypeRegistry;
use crate::compiler::bytecode::ByteCode::{Load, Store};
use crate::compiler::codegen::BytecodeGen;
use crate::error::compilation_error::EmptyRes;
use crate::error::ok;

pub fn implement_functions(registry: &TypeRegistry, bytecode_gen: &mut BytecodeGen) {
    implement_heap_alloc(registry,bytecode_gen).unwrap();
    implement_load_at(registry, bytecode_gen).unwrap();
    implement_store_at(registry, bytecode_gen).unwrap();
    implement_int(registry, bytecode_gen).unwrap();
    implement_float(registry, bytecode_gen).unwrap();
    implement_str(registry, bytecode_gen).unwrap();
    implement_print(registry, bytecode_gen).unwrap();
}

fn implement_int(registry: &TypeRegistry, generator: &mut BytecodeGen) -> EmptyRes {
    generator.fn_start("int_f".to_string());
    generator.f2i();
    generator.ret(1);
    generator.fn_end("int_f".to_string(), registry)?;

    ok()
}

fn implement_float(registry: &TypeRegistry, generator: &mut BytecodeGen) -> EmptyRes {
    generator.fn_start("float_i".to_string());
    generator.i2f();
    generator.ret(1);
    generator.fn_end("float_i".to_string(), registry)?;

    ok()
}

fn implement_str(registry: &TypeRegistry, generator: &mut BytecodeGen) -> EmptyRes {
    generator.fn_start("str_i".to_string());
    generator.i2str();
    generator.ret(1);
    generator.fn_end("str_i".to_string(), registry)?;

    generator.fn_start("str_f".to_string());
    generator.f2str();
    generator.ret(1);
    generator.fn_end("str_f".to_string(), registry)?;

    ok()
}

fn implement_print(registry: &TypeRegistry, generator: &mut BytecodeGen) -> EmptyRes {
    generator.fn_start("print_s".to_string());
    generator.print();
    generator.ret(0);
    generator.fn_end("print_s".to_string(), registry)?;

    generator.fn_start("print_i".to_string());
    generator.i2str();
    generator.print();
    generator.ret(0);
    generator.fn_end("print_i".to_string(), registry)?;

    generator.fn_start("print_f".to_string());
    generator.f2str();
    generator.print();
    generator.ret(0);
    generator.fn_end("print_f".to_string(), registry)?;

    ok()
}

fn implement_load_at(registry: &TypeRegistry, generator: &mut BytecodeGen) -> EmptyRes {
    generator.fn_start("address$__load_at_i".to_string());
    generator.load_var_offset()?;
    generator.ret(1);
    generator.fn_end("address$__load_at_i".to_string(), registry)?;

    ok()
}

fn implement_store_at(registry: &TypeRegistry, generator: &mut BytecodeGen) -> EmptyRes {
    generator.fn_start("address$__store_at_i?".to_string());

    generator.swap();
    generator.store_internal(|i| { Store(i) });
    generator.swap();
    unsafe {generator.push_own_insn(Load(0));}
    generator.store_var_offset()?;
    generator.ret(0);
    generator.fn_end("address$__store_at_i?".to_string(), registry)?;

    ok()
}

fn implement_heap_alloc(registry: &TypeRegistry, generator: &mut BytecodeGen) -> EmptyRes {
    generator.fn_start("__heap_alloc_i".to_string());

    generator.dyn_heap_alloc();

    generator.ret(1);
    generator.fn_end("__heap_alloc_i".to_string(), registry)?;


    ok()
}