use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemTrait, Token, parse_macro_input, punctuated::Punctuated};

#[proc_macro_attribute]
pub fn components(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input_trait = parse_macro_input!(input as ItemTrait);

    let target_traits: Punctuated<Ident, Token![,]> =
        parse_macro_input!(args with Punctuated::<Ident, Token![,]>::parse_terminated);

    let mut impl_blocks = Vec::new();

    for target_trait in target_traits {
        let method_name: Ident =
            syn::parse_str(&target_trait.to_string().to_case(Case::Snake)).unwrap();

        let new_method = syn::parse_quote! {
            #[doc = "Returns a reference to the `"]
            #[doc = stringify!(#target_trait)]
            #[doc = "` implementation. Use unit type for default implementation."]
            fn #method_name(&self) -> &dyn #target_trait {
                &()
            }
        };

        input_trait.items.push(new_method);

        let impl_block = quote! {
            impl #target_trait for () {}
        };
        impl_blocks.push(impl_block);
    }

    quote! {
        #input_trait

        #(#impl_blocks)*
    }
    .into()
}
