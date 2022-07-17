# CPS (AKA macro variables)

TLDR:
```rust
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

## Why?

Macro execution order is confusing. Because each macro is passed a token tree, macros execute outside-in. For example:

```rust
macro_rules! dog {
    () => {
        woof
    };
}

fn main() {
    println!("{}", stringify!(dog!())); // Prints "dog!()", not "woof"
}
```

Reading the above code as if macros are classical functions, you may expect this program to print `woof`. However unfortunately it prints `dog!()`, as if `println!` expands its macros while `stringify!` does not. This makes macros hard to maintain.

[The Little Book of Macros](https://veykril.github.io/tlborm/decl-macros/patterns/callbacks.html) describes *callbacks*, where a macro takes as an argument the next macro to execute. This leads to the following improved version of the above example:

```rust
macro_rules! dog {
    ($cont:ident) => {
        $cont!(woof)
    };
}

fn main() {
    println!("{}", dog!(stringify)); // Prints "woof"
}
```

While now having the correct behaviour, this is difficult to maintain as the flow of execution is confusing. Using CPS instead we get:

```rust
#[cps]
macro_rules! dog {
    () => {
        woof
    };
    
    (str) => 
    let $x::tt = dog!() in
    {
        stringify!($x)
    };
}

fn main() {
    println!("{}", dog!(str)); // Prints "woof"
}
```

