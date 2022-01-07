use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput};

#[macro_use]
extern crate quote;
extern crate proc_macro;

/// The Migration macro is applied to a type to automatically implement the MigrationName trait.
#[proc_macro_derive(Migration)]
pub fn derive_migrator_macro(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    if !matches!(data, Data::Struct(_)) {
        panic!("The 'Migrator' macro can only be used on structs!");
    }

    let value = ident.to_string();
    quote! {
        impl sea_migrations::MigrationName for #ident {
            fn name(&self) -> &'static str {
                #value
            }
        }
    }
    .into()
}
