use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};


#[proc_macro_attribute]
pub fn expression(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = {
        quote! {
                #[derive(Debug)]
                #input

                impl #impl_generics Expression for #name #ty_generics #where_clause {
                }
            }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn statement(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = {
        quote! {
                #[derive(Debug)]
                #input

                impl #impl_generics Statement for #name #ty_generics #where_clause {
                }
            }
    };

    TokenStream::from(expanded)
}