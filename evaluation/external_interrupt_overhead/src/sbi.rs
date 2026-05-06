#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> (usize, usize) {
    let (error, value);
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
        );
    }
    (error, value)
}

// Console Putchar (EID #0x01)
const EID_CONSOLE_PUTCHAR: usize = 0x01;
const FID_CONSOLE_PUTCHAR: usize = 0x0;

/// Puts a character to the console.
pub fn console_putchar(c: usize) {
    sbi_call(EID_CONSOLE_PUTCHAR, FID_CONSOLE_PUTCHAR, c, 0, 0);
}
