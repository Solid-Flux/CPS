#![deny(missing_docs)]

/*!
# CPS
Provides a proc-macro attribute to assist in creation of readable and maintainable macro_rules! macros by generating Continuation Passing Style variations automatically.

### The Problem

Macro execution order is confusing. Because each macro is passed a token tree, macros execute outside-in. For example:

```
macro_rules! dog {
    () => {
        woof
    };
}

fn main() {
    println!("{}", stringify!(dog!())); // Prints "dog!()"
}
```

Reading the above code as if macros are classical functions, you may expect this program to print `woof`. However unfortunately it prints `dog!()`, as if `println!` expands its macros while `stringify!` does not. This makes macros hard to maintain.

### The Old Solution

[The Little Book of Macros](https://veykril.github.io/tlborm/decl-macros/patterns/callbacks.html) describes *callbacks*, where a macro takes as an argument the next macro to execute. This leads to the following improved version of the above example:

```
macro_rules! dog {
    ($cont:ident) => {
        $cont!(woof)
    };
}

fn main() {
    println!("{}", dog!(stringify)); // Prints "woof"
}
```

While now possibly having the correct behaviour, this is still difficult to maintain as the order of execution is confusing.

### The New Solution

This crate offers the following new syntax when defining macros:

```
#[cps] // `dog` must be marked as cps to be used in `stringified_dog`
macro_rules! dog {
    () => {
        woof
    };
}

#[cps]
macro_rules! stringified_dog {
    () =>
    let $foo = dog!() in // !!! NEW SYNTAX !!!
    {
        stringify!($foo)
    };
}

fn main() {
    println!("{}", stringified_dog!()); // Prints "woof"
}
```

## Features

Marking your macro with the `#[cps]` attribute gives you the ability to use recursion:

```
#[cps]
macro_rules! fib {
    () => {
        1
    };

    (x) => {
        1
    };

    (x x $($xs:tt)*) =>
    let $a = fib!(x $($xs)*) in
    let $b = fib!($($xs)*) in
    {
        $a + $b
    };
}

fn main() {
    println!("{}", fib!(x x x x x x x)); // 7 x's -> prints 21
}
```

Note that this example is highly inefficient and is for demonstration purposes only. It could also be implemented naively without the use of CPS, and is only present to illustrate the syntax.

`#[cps]` also gives your macro the ability to be used in other cps macros:

```
#[cps]
macro_rules! foo {
    ($a:ident) => {
        Hello $a
    };
}

#[cps]
macro_rules! bar {
    () =>
    let $foo = foo!(World) in
    {
        stringify!($foo)
    };
}

fn main() {
    println!("{}", bar!()); // Prints "Hello World"
}
```

CPS makes writing macros easier, which many people think is a terrible idea. Macros are hard, difficult to maintain, and you should always consider writing a proc-macro instead. This library aims to make the macros that you *do* write more maintainable. Please recurse responsibly.

## Usage Notes

CPS converts iteration into recursion. Therefore when using this library you may reach the recursion limit (128 at the time of writing). You can raise this using `#![recursion_limit = "1024"]` but your build times may suffer.

Any macro `let` expression must have a macro on the right-hand side that was marked as `#[cps]`. The following example will not work!

```
#[cps]
macro_rules! foo {
    ($a:ident) => {
        Hello $a
    };
}

#[cps]
macro_rules! bar {
    () =>
    let $foo = foo!(World) in
    let $bar = stringify!($foo) in // Issue: stringify is not a cps macro
    {
        $bar
    };
}
```

## Examples

This macro implements a simple functional `map` operation over a set of expressions:

```
#[cps]
macro_rules! map {
    (@ $f:ident []) => { };

    (@ $f:ident [$head:expr $(, $tail:expr)*]) =>
    let $rest = map!(@ $f [$($tail),]) in
    {
        $f($head), $rest
    };

    ($f:ident [$($list:expr),*]) =>
    let $mapped = map!(@ $f [$($list),]) in
    {
        vec![$mapped]
    };
}

fn double(x: i32) -> i32 {
    x * 2
}

fn main() {
    let mapped = map!(double [1, 2, 3, 4, 5]);
    println!("{}", mapped); // Prints [2, 4, 6, 8, 10]
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
/// This crate provides an attribute macro, [#\[cps\]](crate::cps), that allows far more readable macros
/// to be written:
///
/// ```
/// #[cps]
/// macro_rules! expand_to_larch {
///     () => { larch };
/// }
///
/// #[cps]
/// macro_rules! recognize_tree {
///     (larch) => { println!("#1, the Larch.") };
///     (redwood) => { println!("#2, the Mighty Redwood.") };
///     (fir) => { println!("#3, the Fir.") };
///     (chestnut) => { println!("#4, the Horse Chestnut.") };
///     (pine) => { println!("#5, the Scots Pine.") };
///     ($($other:tt)*) => { println!("I don't know; some kind of birch maybe?") };
/// }
///
/// #[cps]
/// macro_rules! name_a_larch {
///     () =>
///     let $tree = expand_to_larch!() in
///     let $output = recognize_tree!($tree) in
///     {
///         $output
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
