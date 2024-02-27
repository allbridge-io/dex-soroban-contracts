use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parser as _, Ident};

#[proc_macro_derive(Event)]
pub fn derive_soroban_event(input: TokenStream) -> TokenStream {
    let syn::DeriveInput { ident, .. } = syn::parse_macro_input! {input};

    let ident_name = ident.to_string();
    let ident_name = ident_name.as_str();

    quote! {
        impl shared::Event for #ident {
            const EVENT_NAME: &'static str = #ident_name;
        }
    }
    .into()
}

#[proc_macro_derive(SorobanData)]
pub fn derive_soroban_data(input: TokenStream) -> TokenStream {
    let syn::DeriveInput { ident, .. } = syn::parse_macro_input! {input};

    quote! {
        impl shared::soroban_data::SorobanData for #ident {}
    }
    .into()
}

#[proc_macro_derive(SorobanSimpleData)]
pub fn derive_soroban_simple_data(input: TokenStream) -> TokenStream {
    let syn::DeriveInput { ident, .. } = syn::parse_macro_input! {input};

    quote! {
        impl shared::soroban_data::SimpleSorobanData for #ident {}
    }
    .into()
}

fn impl_data_storage_type(ident: &Ident, s: &'static str) -> TokenStream {
    let path = format!("shared::StorageType::{}", s);
    let path: syn::Path = syn::parse_str(path.as_str()).unwrap();

    quote!(
        impl shared::soroban_data::DataStorageType for #ident {
            const STORAGE_TYPE: shared::StorageType = #path;
        }
    )
    .into()
}

#[proc_macro_derive(Temporary)]
pub fn derive_data_storage_type_temporary(input: TokenStream) -> TokenStream {
    let input: syn::ItemStruct = syn::parse(input).unwrap();
    impl_data_storage_type(&input.ident, "Temporary")
}

#[proc_macro_derive(Instance)]
pub fn derive_data_storage_type_instance(input: TokenStream) -> TokenStream {
    let input: syn::ItemStruct = syn::parse(input).unwrap();
    impl_data_storage_type(&input.ident, "Instance")
}

#[proc_macro_derive(Persistent)]
pub fn derive_data_storage_type_persistent(input: TokenStream) -> TokenStream {
    let input: syn::ItemStruct = syn::parse(input).unwrap();
    impl_data_storage_type(&input.ident, "Persistent")
}

#[proc_macro_derive(SymbolKey)]
pub fn derive_symbol_key(input: TokenStream) -> TokenStream {
    let syn::DeriveInput { ident, .. } = syn::parse_macro_input! {input};

    let key = ident.to_string();
    let key = key.as_str();

    if key.len() > 32 {
        return quote! {
            compile_error!("Symbol maximum length is 32 characters");
        }
        .into();
    }

    quote! {
        impl shared::soroban_data::SymbolKey for #ident {
            const STORAGE_KEY: &'static str = #key;
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn extend_ttl_info_instance(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input: syn::ItemStruct = syn::parse(input).unwrap();
    let ident = &input.ident;

    quote!(
        #input

        impl shared::soroban_data::ExtendTtlInfo for #ident {
            const EXTEND_TTL_AMOUNT: u32 = shared::consts::INSTANCE_EXTEND_TTL_AMOUNT;
            const LIFETIME_THRESHOLD: u32 = shared::consts::INSTANCE_LIFETIME_THRESHOLD;
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn extend_ttl_info(args: TokenStream, input: TokenStream) -> TokenStream {
    let input: syn::ItemStruct = syn::parse(input).unwrap();
    let ident = &input.ident;

    let parsed_args =
        syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated.parse(args);

    let parsed_args = parsed_args
        .map_err(|err| -> TokenStream {
            let err = err.to_compile_error();
            quote!( compile_error!(#err); ).into()
        })
        .unwrap();

    if parsed_args.len() != 2 {
        return quote!(
            compile_error!("Received an unexpected number of arguments (2)");
        )
        .into();
    }

    let extend_ttl_amount = parsed_args.first().unwrap();
    let lifetime_threshold = parsed_args.last().unwrap();

    quote!(
        #input

        impl shared::soroban_data::ExtendTtlInfo for #ident {
            const EXTEND_TTL_AMOUNT: u32 = #extend_ttl_amount;
            const LIFETIME_THRESHOLD: u32 = #lifetime_threshold;
        }
    )
    .into()
}
