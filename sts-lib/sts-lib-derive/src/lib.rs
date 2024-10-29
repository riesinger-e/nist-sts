//! Proc macros to help with sts-lib.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{ExprPath, ItemFn};

/// Input to the `use_thread_pool` proc macro.
struct ThreadPoolArg {
    // The name of the thread pool to use.
    variable: ExprPath,
}

impl Parse for ThreadPoolArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let variable = ExprPath::parse(input)?;
        Ok(Self { variable })
    }
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
/// You have to specify the custom thread pool (variable name),
/// like this: `#[use_thread_pool(CUSTOM_THREAD_POOL)]`.
#[proc_macro_attribute]
pub fn use_thread_pool(arg: TokenStream, input: TokenStream) -> TokenStream {
    // Syntax tree for attr
    let attr = syn::parse_macro_input!(arg as ThreadPoolArg);
    let name = attr.variable;

    // Syntax tree for code
    let input = syn::parse_macro_input!(input as ItemFn);

    let ItemFn {
        attrs,
        vis: visibility,
        sig: signature,
        block: body,
    } = input;

    TokenStream::from(quote! {
        #(#attrs)*
        #visibility #signature {
            #name.install(|| #body)
        }
    })
}
