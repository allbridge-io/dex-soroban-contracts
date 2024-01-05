use proc_macro::TokenStream;
use quote::{quote, ToTokens};

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

#[proc_macro_attribute]
pub fn symbol_key(args: TokenStream, input: TokenStream) -> TokenStream {
    let input: syn::ItemStruct = syn::parse(input).unwrap();
    let ident = &input.ident;

    let storage_key: syn::LitStr = syn::parse(args).unwrap();

    quote!(
        #input

        impl shared::soroban_data::SymbolKey for #ident {
            const STORAGE_KEY: soroban_sdk::Symbol = soroban_sdk::symbol_short!(#storage_key);
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn data_storage_type(args: TokenStream, input: TokenStream) -> TokenStream {
    let input: syn::ItemStruct = syn::parse(input).unwrap();
    let ident = &input.ident;

    let storage_type = args.to_string();
    let path: syn::Path =
        syn::parse_str(&format!("shared::StorageType::{}", storage_type)).unwrap();

    (match storage_type.as_str() {
        "Temporary" | "Persistent" | "Instance" => quote!(
            #input

            impl shared::soroban_data::DataStorageType for #ident {
                const STORAGE_TYPE: shared::StorageType = #path;
            }
        ),
        _ => {
            quote!(  compile_error!("Unexpected StorageType, use Temporary/Persistent/Instance");  )
        }
    })
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

    let args = args.into_iter().collect::<Vec<_>>();

    if args.len() != 3 {
        return quote!(
            compile_error!("Received an unexpected number of arguments (3)");
        )
        .into();
    }

    let extend_ttl_amount = TokenStream::from_iter([args[0].clone()]);
    let lifetime_threshold = TokenStream::from_iter([args[2].clone()]);

    let extend_ttl_amount_lit_int =
        syn::parse::<syn::LitInt>(extend_ttl_amount.clone()).map(ToTokens::into_token_stream);
    let extend_ttl_amount_ident =
        syn::parse::<syn::Ident>(extend_ttl_amount.clone()).map(ToTokens::into_token_stream);
    let extend_ttl_amount = extend_ttl_amount_ident.or(extend_ttl_amount_lit_int).unwrap();

    let lifetime_threshold_lit_int =
        syn::parse::<syn::LitInt>(lifetime_threshold.clone()).map(ToTokens::into_token_stream);
    let lifetime_threshold_ident =
        syn::parse::<syn::Ident>(lifetime_threshold.clone()).map(ToTokens::into_token_stream);
    let lifetime_threshold = lifetime_threshold_ident
        .or(lifetime_threshold_lit_int)
        .unwrap();

    quote!(
        #input

        impl shared::soroban_data::ExtendTtlInfo for #ident {
            const EXTEND_TTL_AMOUNT: u32 = #extend_ttl_amount;
            const LIFETIME_THRESHOLD: u32 = #lifetime_threshold;
        }
    )
    .into()
}
