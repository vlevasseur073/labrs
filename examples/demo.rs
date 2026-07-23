//! # Example notebook
//!
//! Demonstrates helpers vs cells.

use labrs::prelude::*;
use serde::{Deserialize, Serialize};

fn double(val: u16) -> u16 {
    2 * val
}

#[derive(Debug, Serialize, Deserialize)]
struct MyStruct {
    a: i32,
    path: String,
    b: bool,
    c: u16,
}

impl MyStruct {
    fn new(a: i32, path: String, b: bool, c: u16) -> Self {
        Self { a, path, b, c }
    }
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
    8
}

#[labrs::cell]
pub fn report(val: &u16) -> String {
    let double_val = double(*val);
    let msg = format!("Double of {val} is {double_val}");
    println!("{msg}");
    msg
}

#[labrs::cell]
pub fn my_struct() -> MyStruct {
    let my_struct = MyStruct::new(12, "/toto/titi".to_string(), false, 1);
    my_struct
}

#[labrs::cell]
pub fn use_my_struct(my_struct: MyStruct) -> i32 {
    println!("{:?}", my_struct);
    0
}
