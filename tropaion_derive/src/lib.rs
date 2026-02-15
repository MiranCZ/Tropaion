use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemStruct};


#[proc_macro_attribute]
pub fn expression(_attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_impl(item, "Expression")
}

#[proc_macro_attribute]
pub fn statement(_attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_impl(item, "Statement")
}

#[proc_macro_attribute]
pub fn ast_type(_attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_impl(item, "AstType")
}

fn generate_impl(item: TokenStream, trait_name: &str) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let trait_ident = format_ident!("{}", trait_name);

    let expanded = {
        quote! {
                #[derive(Debug)]
                #input

                impl #impl_generics #trait_ident for #name #ty_generics #where_clause {
                }
            }
    };

    TokenStream::from(expanded)
}

