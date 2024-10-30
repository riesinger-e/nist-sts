//! Proc macros to help with sts-lib.
//!
//! About the implementation: compare to https://docs.rust-embedded.org/embedonomicon/singleton.html

use proc_macro::TokenStream;
use std::option::IntoIter;
use quote::quote;
use syn::{parse_quote, Ident, ItemFn, ItemStatic, Meta};
use syn::__private::Span;

const THREADPOOL_NAME: &str = "__STS_MACRO_INTERNALS_THREADPOOL";

/// Registers the specified static to be used as the thread pool for use in the [use_thread_pool]
/// macro. This macro must be called exactly once if using [use_thread_pool].
///
/// This macro must be used on a static item of type `std::sync::lazy_lock::LazyLock<rayon::ThreadPool>`.
/// The visibility of the item will always be set to public, and it will not be mut.
///
/// The given static may not use `#[export_name]`
#[proc_macro_attribute]
pub fn register_thread_pool(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ItemStatic);

    let ItemStatic {
        attrs,
        // disregard visibility --> always set to public
        vis: _,
        static_token: _,
        // disregard any mutability
        mutability: _,
        ident,
        colon_token: _,
        // disregard the type and set it manually - if the user tries to use another type, an error
        // will happen because of inference breaking
        ty: _,
        eq_token: _,
        expr,
        semi_token: _,
    } = input;

    // check attributes - attribute export_name must not be used, because it is added by this macro.
    // also need to check if #[used] is already there
    let mut used_already_exists = false;

    for attr in attrs.iter() {        
        match &attr.meta {
            Meta::List(list) if attr.path().is_ident("unsafe") => list
                .parse_nested_meta(|meta| {
                    assert!(!meta.path.is_ident("export_name"), "Attribute 'export_name' must be used by this macro!");
                    Ok(())
                })
                .unwrap(),
            Meta::NameValue(name_value) => {
                assert!(!name_value.path.is_ident("export_name"), "Attribute 'export_name' must be used by this macro!")
            }
            Meta::Path(path) => {
                if path.is_ident("used") {
                    used_already_exists = true;
                }
            },
            _ => (),
        }
    }
    
    let used_attribute: IntoIter<Ident> = if !used_already_exists {
        Some(parse_quote!(used)).into_iter()
    } else {
        None.into_iter()
    };

    TokenStream::from(quote! {
        #(#attrs)*
        #(#[#used_attribute])*
        #[export_name = #THREADPOOL_NAME]
        pub static #ident: ::std::sync::LazyLock<::rayon::ThreadPool> = #expr;
    })
}

/// Use the given thread pool when running the function. This attribute allows using a custom
/// rayon thread pool without explicitly writing it.
///
/// This proc macro should be used for all public API functions that could use rayon, to force
/// usage of the library-specific thread pool instead of the rayon global pool.
///
/// This should be used for all statistical tests.
///
/// ## Usage
///
/// This macro takes no arguments. The threadpool to use is specified via the [register_thread_pool]
/// macro.
#[proc_macro_attribute]
pub fn use_thread_pool(_: TokenStream, input: TokenStream) -> TokenStream {
    // Syntax tree for code
    let input = syn::parse_macro_input!(input as ItemFn);

    let ItemFn {
        attrs,
        vis: visibility,
        sig: signature,
        block: body,
    } = input;
    
    let threadpool_name: Ident = Ident::new(THREADPOOL_NAME, Span::call_site());

    TokenStream::from(quote! {
        #(#attrs)*
        #visibility #signature {
            let body = || #body;
            
            unsafe {
                extern "Rust" {
                    static #threadpool_name: ::std::sync::LazyLock<::rayon::ThreadPool>;
                }
                #threadpool_name.install(body)
            }
        }
    })
}
