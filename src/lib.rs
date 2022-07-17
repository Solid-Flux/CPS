#![deny(missing_docs)]

/*!
# CPS
Provides a proc-macro attribute to assist in creation of readable and maintainable macro_rules! macros by generating Continuation Passing Style variations automatically.

This crate offers the following new syntax when defining macros:

```
use cps::cps;

#[cps]
macro_rules! foo {
    () => { BaseCase };

    (bar) =>
    let $x:tt = foo!() in
    {
        stringify!($x)
    };
}


fn main() {
    assert_eq!(foo!(bar), "BaseCase");
}
```

Macros-by-example are hard, difficult to maintain, and you should always consider writing a proc-macro instead. This library aims to make the macros that you *do* write more maintainable. Please recurse responsibly.

## Usage Notes

CPS converts iteration into recursion. Therefore when using this library you may reach the recursion limit (128 at the time of writing). You can raise this using `#![recursion_limit = "1024"]` but your build times may suffer.

Any macro `let` expression must have a macro on the right-hand side that was marked as `#[cps]`. The following example will not work!

```
# use cps::cps;

#[cps]
macro_rules! foo {
    () => { BaseCase };

    (bar) =>
    let $x:tt = foo!() in
    let $y:tt = stringify!($x) in // Issue: stringify is not a cps macro
    {
        $y
    };
}
```
 */

mod cps_macro;
mod parse;

use proc_macro::TokenStream;
use syn::{ItemMacro, parse_macro_input};


/// Manipulates a macro_rules! definition to add extended syntax to help in creating readable macros.
///
/// Macro execution order is tricky. For example, the output of the following code goes against our
/// intuition of how functions should work:
///
/// ```
/// macro_rules! expand_to_larch {
///     () => { larch };
/// }
///
/// macro_rules! recognize_tree {
///     (larch) => { println!("#1, the Larch.") };
///     (redwood) => { println!("#2, the Mighty Redwood.") };
///     (fir) => { println!("#3, the Fir.") };
///     (chestnut) => { println!("#4, the Horse Chestnut.") };
///     (pine) => { println!("#5, the Scots Pine.") };
///     ($($other:tt)*) => { println!("I don't know; some kind of birch maybe?") };
/// }
///
/// fn main() {
///     recognize_tree!(expand_to_larch!()); // Prints "I don't know; some kind of birch maybe?"
/// }
/// ```
///
/// [The Little Book of Rust Macros][tlborm] (where the above example comes from) outlines *callbacks* -
/// a macro pattern that allows macro execution order to be specified:
///
/// ```
/// # macro_rules! recognize_tree {
/// #    (larch) => { println!("#1, the Larch.") };
/// #    (redwood) => { println!("#2, the Mighty Redwood.") };
/// #    (fir) => { println!("#3, the Fir.") };
/// #    (chestnut) => { println!("#4, the Horse Chestnut.") };
/// #    (pine) => { println!("#5, the Scots Pine.") };
/// #    ($($other:tt)*) => { println!("I don't know; some kind of birch maybe?") };
/// # }
///
/// macro_rules! call_with_larch {
///     ($callback:ident) => { $callback!(larch) };
/// }
///
/// fn main() {
///     call_with_larch!(recognize_tree); // Correctly prints "#1, the Larch."
/// }
/// ```
///
/// This syntax, while powerful, soon becomes confusing.
///
/// This macro allows far more readable macros
/// to be written:
///
/// ```
/// # use cps::cps;
///
/// #[cps]
/// macro_rules! expand_to_larch {
///     () => { larch };
/// }
///
/// #[cps]
/// macro_rules! recognize_tree {
///     (larch) => { println!("#1, the Larch.") };
///     // ...
///     ($($other:tt)*) => { println!("I don't know; some kind of birch maybe?") };
/// }
///
/// #[cps]
/// macro_rules! name_a_larch {
///     () =>
///     let $tree:tt = expand_to_larch!() in
///     {
///         recognize_tree!($tree)
///     };
/// }
///
/// fn main() {
///     name_a_larch!(); // Prints "#1, the Larch."
/// }
/// ```
///
/// [tlborm]: https://veykril.github.io/tlborm/decl-macros/patterns/callbacks.html
#[proc_macro_attribute]
pub fn cps(attr: TokenStream, item: TokenStream) -> TokenStream {
    let m = parse_macro_input!(item as ItemMacro);

    TokenStream::from(cps_macro::impl_cps(proc_macro2::TokenStream::from(attr), m))
}
