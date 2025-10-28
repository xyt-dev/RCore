//! SBI call wrappers

use core::arch::asm;

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_SHUTDOWN: usize = 8;

/// general sbi call
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x16") 0,
            in("x17") which,
        );
    }
    ret
}

/// use sbi call to set timer
pub fn set_timer(timer: usize) {
    sbi_call(SBI_SET_TIMER, timer, 0, 0);
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

/// use sbi call to shutdown the kernel
pub fn shutdown() -> ! {
        unsafe {
            // asm!(
                // "sw {0}, 0({1})",
                // in(reg)0x5555, in(reg)(0x100000)
            // );

            // For the case that the QEMU exit attempt did not work, transition into an infinite
            // loop. Calling `panic!()` here is unfeasible, since there is a good chance
            // this function here is the last expression in the `panic!()` handler
            // itself. This prevents a possible infinite loop.
            // loop {
            //     asm!("wfi", options(nomem, nostack));
            // }
        }
    // sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}
