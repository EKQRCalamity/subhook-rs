use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Attribute macro to make a Rust function hookable.
///
/// This macro automatically adds the necessary attributes to make a function
/// suitable for hooking:
/// - `#[inline(never)]` - Prevents the compiler from inlining the function
/// - `extern "C"` - Uses a stable ABI for predictable calling conventions
///
/// # Examples
///
/// ```
/// use subhook_rs::hook;
///
/// #[hook]
/// fn add(a: u32, b: u32) -> u32 {
///     a + b
/// }
/// ```
///
/// This expands to:
///
/// ```
/// #[inline(never)]
/// extern "C" fn add(a: u32, b: u32) -> u32 {
///     a + b
/// }
/// ```
///
/// # Visibility
///
/// The macro preserves visibility modifiers:
///
/// ```
/// use subhook_rs::hook;
///
/// #[hook]
/// pub fn my_function() -> i32 {
///     42
/// }
/// ```
///
/// # Limitations
///
/// - Does not support generic functions (generics create multiple function versions)
/// - Does not support async functions
/// - Does not support const functions
/// - The function will use C calling convention, which may have slight performance implications
#[proc_macro_attribute]
pub fn hook(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    // Reject async functions
    if input.sig.asyncness.is_some() {
        return syn::Error::new_spanned(
            &input.sig.asyncness,
            "#[hook] cannot be applied to async functions",
        )
        .to_compile_error()
        .into();
    }

    // Reject const functions
    if input.sig.constness.is_some() {
        return syn::Error::new_spanned(
            &input.sig.constness,
            "#[hook] cannot be applied to const functions",
        )
        .to_compile_error()
        .into();
    }

    if !input.sig.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &input.sig.generics,
            "#[hook] cannot be applied to generic functions",
        )
        .to_compile_error()
        .into();
    }

    input.sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern::default(),
        name: Some(syn::LitStr::new("C", proc_macro2::Span::call_site())),
    });

    let inline_never: syn::Attribute = syn::parse_quote! {
        #[inline(never)]
    };
    input.attrs.push(inline_never);

    TokenStream::from(quote! {
        #input
    })
}
