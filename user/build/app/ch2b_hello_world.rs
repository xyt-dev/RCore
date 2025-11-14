#![no_std]
#![no_main]

extern crate user_lib;

/// 正确输出：
/// Hello world from user mode program!

#[no_mangle]
fn main() -> i32 {
    user_lib::println!("Hello, world from user mode program!!");
    0
}