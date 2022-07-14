use cps::cps;

#[cps]
macro_rules! macro1 {
    () => { BaseCase };

    (stringify) =>
    let $x:tt = macro1!() in
    {
        stringify!($x)
    };
}


#[test]
fn stringify_order_single_call() {
    assert_eq!(macro1!(stringify), "BaseCase");
}
