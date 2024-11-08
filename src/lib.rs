#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![doc = include_str!("../README.md")]

use block::{Checked, MathBlock, Overflowing, Propagating, Saturating};
use quote::quote;
use syn::{parse_macro_input, visit_mut::VisitMut, Block};
use visitor::MathBlockVisitor;

mod block;
mod expr;
mod visitor;

/// Defines a block of code where all mathematical operations are performed using checked methods.
///
/// If any operation overflows, it will panic with a corresponding error message.
///
/// # Example
///
/// ```rust
/// use overf::checked;
///
/// fn main() {
///     checked! {
///         let a = 10usize + 5usize;
///         let b = 20usize - 10usize;
///         let c = 3usize * 7usize;
///     }
/// }
/// ```
#[proc_macro]
pub fn checked(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand::<Checked>(input)
}

/// Defines a block of code where all mathematical operations use overflowing methods.
///
/// When an operation overflows, it will not panic; instead, it will return the result of the operation, wrapping around if necessary.
///
/// # Example
///
/// ```rust
/// use overf::overflowing;
///
/// fn main() {
///     overflowing! {
///         let a = 10usize + 5usize;
///         let b = 200usize - 300usize; // Overflows
///     }
/// }
/// ```
#[proc_macro]
pub fn overflowing(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand::<Overflowing>(input)
}

/// Defines a block of code where all mathematical operations use saturating methods.
///
/// When an operation would overflow, it will instead return the maximum (or minimum) value of the type.
///
/// # Example
///
/// ```rust
/// use overf::saturating;
///
/// fn main() {
///     saturating! {
///         let a = usize::MAX + 1; // Saturates to usize::MAX
///         let b = usize::MIN - 1; // Saturates to usize::MIN
///     }
/// }
/// ```
#[proc_macro]
pub fn saturating(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand::<Saturating>(input)
}

/// Defines a block of code where all mathematical operations use checked methods.
/// If any operation results in an overflow, it will return `None`, propagating the error using the `?` operator.
///
/// This macro is useful when you want to handle potential overflows in a concise way, allowing the calling function
/// to manage the error or terminate early.
///
/// # Example
///
/// ```rust
/// use overf::propagating;
///
/// fn example() -> Option<usize> {
///     propagating! {
///         // Returns `None` if any operation fails.
///         let a = 10usize + 5usize;
///         let b = 20usize - 10usize;
///         let c = 3usize * 7usize;
///         let d = 21usize / 3usize;
///         let e = a + b;
///         Some(e)
///     }
/// }
/// ```
#[proc_macro]
pub fn propagating(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand::<Propagating>(input)
}

/// Resets the overflow behavior to the default behavior of Rust.
///
/// This is useful when you want to exit a block with custom overflow handling and revert to the standard behavior.
///
/// # Example
///
/// ```rust
/// use overf::{checked, default};
///
/// fn main() {
///     checked! {
///         let a = 10usize + 5usize; // checked
///         
///         default! {
///             let b = a + 1000; // Uses default behavior (may panic or overflow)
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn default(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    input
}

fn expand<B: MathBlock>(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input2 = proc_macro2::TokenStream::from(input);
    let input = proc_macro::TokenStream::from(quote! { { #input2 } });
    let block = parse_macro_input!(input as Block);
    match try_expand::<B>(block) {
        Ok(res) => res.into(),
        Err(err) => {
            let error = err.to_compile_error();
            quote! {
                #error
            }
            .into()
        }
    }
}

fn try_expand<B: MathBlock>(mut block: Block) -> syn::Result<proc_macro2::TokenStream> {
    MathBlockVisitor::<B>::new().visit_block_mut(&mut block);
    let mut res = quote! {};
    for stmt in block.stmts {
        res.extend(quote! { #stmt });
    }
    Ok(res)
}
