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

// Base Extension (EID #0x10)
const EID_BASE: usize = 0x10;
const FID_GET_SPEC_VERSION: usize = 0x0;

// Timer Extension (EID #0x54494D45)
const EID_TIME: usize = 0x54494D45;
const FID_SET_TIMER: usize = 0x0;

// Console Putchar (EID #0x01)
const EID_CONSOLE_PUTCHAR: usize = 0x01;
const FID_CONSOLE_PUTCHAR: usize = 0x0;

/// Gets the SBI specification version.
pub fn get_spec_version() -> usize {
    sbi_call(EID_BASE, FID_GET_SPEC_VERSION, 0, 0, 0).1
}

/// Sets the timer.
pub fn set_timer(stime_value: u64) {
    sbi_call(EID_TIME, FID_SET_TIMER, stime_value as usize, 0, 0);
}

/// Puts a character to the console.
pub fn console_putchar(c: usize) {
    sbi_call(EID_CONSOLE_PUTCHAR, FID_CONSOLE_PUTCHAR, c, 0, 0);
}
