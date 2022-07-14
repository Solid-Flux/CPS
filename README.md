# CPS (AKA macro variables)

TLDR:
```rust
use cps::cps;

#[cps]
macro_rules! foo {
    () => { BaseCase };

    (bar) =>
    let $x:tt = macro1!() in
    {
        stringify!($x)
    };
}


fn main() {
    assert_eq!(foo!(bar), "BaseCase");
}
```

## Why?

This is the start of something awful and I will regret my contribution to the rust community immensely if this is a successful crate.