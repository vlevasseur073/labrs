//! # Example notebook
//!
//! Demonstrates helpers vs cells.

use labrs::prelude::*;

fn double(val: u16) -> u16 {
    10 * val
}

fn foo() {
    let mut a = 10;
    let b = &mut a;
    *b = *b + 10;
    println!("a = {a}");
}

#[labrs::cell]
pub fn use_foo() -> i32 {
    foo();
    0
}

#[labrs::markdown]
pub const intro: &str = "# labrs example\n\n`double` is a helper. `val` and `report` are cells.";

#[labrs::cell]
pub fn greeting() -> String {
    "Hello from labrs!".to_string()
}

#[labrs::cell]
pub fn process(greeting: &String) -> String {
    format!("Processed: {greeting}")
}

#[labrs::cell]
pub fn val() -> u16 {
    4
}

#[labrs::cell]
pub fn report(val: &u16) -> String {
    let double_val = double(*val);
    let msg = format!("Double of {val} is {double_val}");
    println!("{msg}");
    msg
}
