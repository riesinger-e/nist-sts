//! Proc macros to help with sts-lib.
//!
//! About the implementation: compare to https://docs.rust-embedded.org/embedonomicon/singleton.html

use proc_macro::TokenStream;
use std::option::IntoIter;
use quote::quote;
use syn::{parse_quote, Attribute, Expr, Ident, ItemFn, Meta, Token};
use syn::__private::Span;
use syn::parse::{Parse, ParseStream};

/// The thread pool to be registered: `static POOL = LazyLock::new(|| ThreadpoolBuilder::new().build().unwrap());`.
struct ThreadpoolItem {
    pub attrs: Vec<Attribute>,
    pub _static_token: Token![static],
    pub ident: Ident,
    pub _eq_token: Token![=],
    pub expr: Box<Expr>,
    pub _semi_token: Token![;],
}

impl Parse for ThreadpoolItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ThreadpoolItem {
            attrs: input.call(Attribute::parse_outer)?,
            _static_token: input.parse()?,
            ident: input.parse()?,
            _eq_token: input.parse()?,
            expr: input.parse()?,
            _semi_token: input.parse()?,
        })
    }
}

/// Returns the export name of the given threadpool
fn threadpool_name() -> String {
    const THREADPOOL_NAME: &str = "__STS_MACRO_INTERNALS_THREADPOOL";
    let crate_name = std::env::var("CARGO_PKG_NAME")
        .expect("CARGO_PKG_NAME must be set to the name of the calling crate")
        .replace('-', "_");

    format!("{THREADPOOL_NAME}_{crate_name}")
}

/// Registers the specified static to be used as the thread pool for use in the [use_thread_pool]
/// macro. This macro must be called exactly once if using [use_thread_pool].
///
/// This macro must be used on a static item of type `std::sync::lazy_lock::LazyLock<rayon::ThreadPool>`.
/// The visibility of the item must not be set, and the item must not be mut.
///
/// The given static may not use `#[export_name]`.
///
/// This macro may only be called once per crate.
///
/// Example:
/// ```ignore
/// use sts_lib_derive::register_thread_pool;
/// use rayon::ThreadPoolBuilder;
/// use std::sync::OnceLock;
///
/// register_thread_pool! {
///     static THREAD_POOL = LazyLock::new(|| ThreadPoolBuilder::new().build().unwrap());
/// }
/// ```
#[proc_macro]
pub fn register_thread_pool(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ThreadpoolItem);

    let ThreadpoolItem {
        attrs,
        _static_token: _,
        ident,
        _eq_token: _,
        expr,
        _semi_token: _,
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

    let threadpool_name = threadpool_name();

    let used_attribute: IntoIter<Ident> = if !used_already_exists {
        Some(parse_quote!(used)).into_iter()
    } else {
        None.into_iter()
    };

    TokenStream::from(quote! {
        #(#attrs)*
        #(#[#used_attribute])*
        #[export_name = #threadpool_name]
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
    
    let threadpool_name: Ident = Ident::new(&threadpool_name(), Span::call_site());

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
