use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType::{Float, FunctionType, FunctionsType, GenericType, Int, StructType, UnknownType, Void};
use crate::ast::ast_type::{AstType, MemberInfo};
use ordermap::OrderMap;
use std::collections::HashMap;

pub fn get_injected_function_identifiers() -> Vec<(&'static str, u32)> {
    vec![
        ("int_f", 1),
        ("float_i", 1),
        ("__heap_alloc_i", 1),
        ("address$__load_at_i", 2),
        ("address$__store_at_i?", 3)
    ]
}

pub fn get_injected_functions(registry: &mut TypeRegistry) -> Vec<AstType> {
    vec![
        heap_alloc_func(registry),
        int_func(registry),
        float_func(registry)
    ]
}


pub fn get_injected_structs(registry: &mut TypeRegistry) -> Vec<AstType> {
    vec![
        address_struct(registry)
    ]
}

/// fn __heap_alloc(size: int) -> address;
fn heap_alloc_func(registry: &mut TypeRegistry) -> AstType {
    let return_type = address_struct(registry);

    FunctionType {
        name: "__heap_alloc".to_string(),
        generics: OrderMap::new(),
        params: vec![registry.register(Int)],
        return_type: registry.register(return_type)
    }
}

fn int_func(registry: &mut TypeRegistry) -> AstType {
    FunctionType {
        name: "int".to_string(),
        generics: OrderMap::new(),
        params: vec![registry.register(Float)],
        return_type: registry.register(Int)
    }
}

fn float_func(registry: &mut TypeRegistry) -> AstType {
    FunctionType {
        name: "float".to_string(),
        generics: OrderMap::new(),
        params: vec![registry.register(Int)],
        return_type: registry.register(Float)
    }
}

///
/// struct address() {
///     fn __load_at(off: int) -> $unknown$;
///     fn __store_at<T>(off: int, T value);
/// }
fn address_struct(registry: &mut TypeRegistry) -> AstType {
    let mut children = HashMap::new();

    {
        let load_at = FunctionType {
            name: "__load_at".to_string(),
            generics: OrderMap::new(),
            params: vec![registry.register(Int)],
            return_type: registry.register(UnknownType)
        };

        let funcs = FunctionsType {
            name: "__load_at".to_string(),
            overloads: vec![registry.register(load_at)],
        };

        children.insert("__load_at".to_string(), MemberInfo(
            registry.register(funcs),
            "__load_at".to_string(),
            1
        ));
    }

    {
        let mut generics = OrderMap::new();

        generics.insert("T".to_string(), registry.register(UnknownType));

        let generic_type = registry.register(GenericType {name: "T".to_string()});

        // let store_at = FunctionType {
        //     name: "__store_at".to_string(),
        //     generics,
        //     params: vec![registry.register(Int), generic_type],
        //     return_type: registry.register(Void)
        // };
        let store_at = FunctionType {
            name: "__store_at".to_string(),
            generics: OrderMap::new(),
            params: vec![registry.register(Int), registry.register(UnknownType)],
            return_type: registry.register(Void)
        };


        let funcs = FunctionsType {
            name: "__store_at".to_string(),
            overloads: vec![registry.register(store_at)],
        };


        children.insert("__store_at".to_string(), MemberInfo(
            registry.register(funcs),
            "__store_at".to_string(),
            2
        ));
    }

    StructType {
        name: "address".to_string(),
        generics: OrderMap::new(),
        fields: vec![],
        children
    }
}
