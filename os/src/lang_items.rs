//! The panic handler

use crate::sbi::shutdown;
use core::panic::PanicInfo;

/// old panic handler
// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     if let Some(location) = info.location() {
//         println!(
//             "[kernel] Panicked at {}:{} {}",
//             location.file(),
//             location.line(),
//             info.message().unwrap()
//         );
//     } else {
//         println!("[kernel] Panicked: {}", info.message().unwrap());
//     }
//     shutdown()
// }

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    if let Some(location) = panic_info.location() {
        println!(
            "panic occurred in file '{}' at line {}",
            location.file(),
            location.line(),
        );
    } else {
        println!("    Panic message: {}", panic_info.message());
    }
    shutdown();
}