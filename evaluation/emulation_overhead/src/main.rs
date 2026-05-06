#![no_std]
#![no_main]

#[macro_use]
mod macros;
mod sbi;

use core::arch::asm;
use core::panic::PanicInfo;

// --- Timer Hardware Constants ---
const TIMER_BASE: usize = 0x5184_0000;
const APBTMR_N_LOAD_COUNT: usize = 0x00;
const APBTMR_N_CURRENT_VALUE: usize = 0x04;
const APBTMR_N_CONTROL: usize = 0x08;

const APBTMR_CONTROL_ENABLE: u32 = 1 << 0;
const APBTMR_CONTROL_MODE_PERIODIC: u32 = 1 << 1;
const APBTMR_MAX_CNT: u32 = 0xFFFFFFFF;

// Frequency is 24MHz (24 cycles per microsecond)
const TICKS_PER_US: u64 = 24;

unsafe extern "C" {
    /// Stack top (defined in linker script)
    pub static _top_stack: u8;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop {
        riscv::asm::wfi();
    }
}

/// Initialize the timer for performance measurement.
pub fn init_timer() {
    let timer_ptr = TIMER_BASE as *mut u32;
    unsafe {
        // Set to max value.
        timer_ptr
            .byte_add(APBTMR_N_LOAD_COUNT)
            .write_volatile(APBTMR_MAX_CNT);
        // Periodic mode, enabled, interrupts disabled.
        let control_val = APBTMR_CONTROL_ENABLE | APBTMR_CONTROL_MODE_PERIODIC;
        timer_ptr
            .byte_add(APBTMR_N_CONTROL)
            .write_volatile(control_val);
    }
}

#[inline]
fn reset_counter() {
    let timer_ptr = TIMER_BASE as *mut u32;
    unsafe {
        // Set to max value.
        timer_ptr
            .byte_add(APBTMR_N_LOAD_COUNT)
            .write_volatile(APBTMR_MAX_CNT);
    }
}

#[inline]
fn get_current_time() -> u64 {
    const RTC_BASE: *const u32 = 0x101000 as *const u32;

    if cfg!(feature = "qemu") {
        unsafe {
            let low = RTC_BASE.add(0).read_volatile();
            let high = RTC_BASE.add(1).read_volatile();
            ((high as u64) << 32) | (low as u64)
        }
    } else {
        let timer_ptr = TIMER_BASE as *mut u32;
        unsafe {
            let current = timer_ptr.byte_add(APBTMR_N_CURRENT_VALUE).read_volatile();
            // Convert down-counter to elapsed upward cycles.
            (APBTMR_MAX_CNT - current) as u64 / TICKS_PER_US
        }
    }
}

#[inline]
fn show_ratio_average(percentage: &'static str, data: u64) {
    if cfg!(feature = "qemu") {
        println!(
            "{} Ratio Average: {} ns (= {} us)",
            percentage,
            data / 10,
            data / 10 / 1000
        );
    } else {
        println!("{} Ratio Average: {} ticks (us)", percentage, data / 10);
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub unsafe extern "C" fn _start() -> ! {
    unsafe {
        asm!(
            // Initialize the stack pointer.
            "la sp, {stack_top}",
            // Jump to the main function.
            "j main",
            stack_top = sym _top_stack,
            options(noreturn)
        );
    }
}

/// Measure instruciton cycle.
fn rdcycle() {
    // unsafe {
    //     let (mut rd, rs1, rs2): (u8, u8, u8) = (0, 0b1111_1011, 2);
    //     asm!(
    //         "bset {rd}, {rs1}, {rs2}",
    //         rd = out(reg) rd,
    //         rs1 = in(reg) rs1,
    //         rs2 = in(reg) rs2,
    //     );
    //     assert_eq!(rd, 0b1111_1111);
    // }

    // let (before_cycle, after_cycle): (u32, u32) = (0, 0);
    // unsafe {
    //     asm!("rdcycle {}", out(reg) before_cycle);
    // }
    // unsafe {
    //     asm!("csrwi ssp, 31");
    // }
    // unsafe {
    //     asm!("rdcycle {}", out(reg) after_cycle);
    // }
    // println!("cycle = {}", after_cycle - before_cycle);
    // println!("done.");

    println!("bset");
    // let mut sum: u64 = 0;
    for _i in 0..110 {
        let diff: u64;
        unsafe {
            asm!(
                "rdcycle t0",
                ".insn 0x29de13b3",
                "rdcycle t1",
                "sub {out}, t1, t0",
                out = out(reg) diff,
                options(nomem, nostack)
            );
        }
        // if i >= 10 {
        //     sum += diff;
        // }

        println!("cycle = {}", diff);
    }
    // println!("ave = {:.3}", sum as f64 / 100.0);
    println!("----------------------------------");

    println!("ssp");
    // let mut sum: u64 = 0;
    for _i in 0..110 {
        let diff: u64;
        unsafe {
            asm!(
                "rdcycle t0",
                "csrwi ssp, 31",
                "rdcycle t1",
                "sub {out}, t1, t0",
                out = out(reg) diff,
                options(nomem, nostack)
            );
        }
        // if i >= 10 {
        //     sum += diff;
        // }

        println!("cycle = {}", diff);
    }
    // println!("ave = {:.3}", sum as f64 / 100.0);
}

/// Measure emulation time
/// (Optional) disabling m-mode illegal instruciton trap.
fn time(deleg: bool) {
    println!(
        "--- Time Benchmark {} Start ---",
        if deleg { "(mtrap disabled)" } else { "" }
    );
    println!("[Latency Test] Measuring cycles per instruction...");
    let mut min_diff = u64::MAX;
    let mut max_diff = 0;

    // deleg to hypervisor
    let (err, _) = sbi::deleg_illegal_insn(deleg);
    if err != 0 {
        panic!("undefined ecall");
    };

    for _ in 0..1000 {
        let diff: u64;
        unsafe {
            asm!(
                "rdcycle t0",
                ".insn 0x29de13b3",
                "rdcycle t1",
                "sub {out}, t1, t0",
                out = out(reg) diff,
                options(nomem, nostack)
            );
        }
        if diff < min_diff {
            min_diff = diff;
        }
        if diff > max_diff {
            max_diff = diff;
        }
    }
    println!("Result: Min={} cycles, Max={} cycles", min_diff, max_diff);

    println!("[Throughput Test] Running 1,000,000 iterations...");

    let start_time = get_current_time();

    // 1000 * 10 + 1000 * 10 * 99 = 1000000
    for _ in 0..100 {
        execute_10_units!(".insn 0x29de13b3", ".insn 0x29de13b3", 1000);
    }

    let end_time = get_current_time();
    let elapsed_ticks = end_time.saturating_sub(start_time);

    // handle in M-mode
    let (err, _) = sbi::deleg_illegal_insn(false);
    if err != 0 {
        panic!("undefined ecall");
    };

    if cfg!(feature = "qemu") {
        println!(
            "Elapsed Time: {} ns (= {} us)",
            elapsed_ticks,
            elapsed_ticks / 1000
        );
    } else {
        println!("Elapsed Time (mtime ticks): {} us", elapsed_ticks);
    }
}

#[allow(named_asm_labels)]
fn mixed_ratio_benchmark() {
    println!("--- Mixed Ratio 1 Million Instructions Benchmark (10-run average) ---");

    // deleg to hypervisor
    let (err, _) = sbi::deleg_illegal_insn(true);
    if err != 0 {
        panic!("undefined ecall");
    };

    // --- 1% Ratio ---
    let mut sum_1pct: u64 = 0;
    unsafe {
        asm!("begin_1pct:");
    }
    for _ in 0..10 {
        let start = get_current_time();
        // 100 * 10 * 10 * 100 = 1000000
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        for _ in 0..99 {
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 100);
        }
        let end = get_current_time();
        sum_1pct += end.saturating_sub(start);
    }
    unsafe {
        asm!("end_1pct:");
    }
    show_ratio_average("1%", sum_1pct);

    // --- 0.1% Ratio ---
    unsafe {
        asm!("begin_01pct:");
    }
    let mut sum_01pct: u64 = 0;
    for _ in 0..10 {
        let start = get_current_time();
        // 1000 * 10 + 1000 * 10 * 99 = 1000000
        execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 1000);
        for _ in 0..99 {
            execute_10_units!(".insn 0x29de13b3", "add t2, t3, t4", 1000);
        }
        let end = get_current_time();
        sum_01pct += end.saturating_sub(start);
    }
    unsafe {
        asm!("end_01pct:");
    }
    show_ratio_average("0.1%", sum_01pct);

    // --- 0.01% Ratio ---
    let mut sum_001pct: u64 = 0;
    unsafe {
        asm!("begin_001pct:");
    }
    for _ in 0..10 {
        let start = get_current_time();
        // 10000 + 10000 * 99 = 1000000
        unsafe {
            asm!(unroll_unit_str!("add t2, t3, t4", "add t2, t3, t4", 1000));
            asm!(unroll_unit_str!("add t2, t3, t4", "add t2, t3, t4", 1000));
            asm!(unroll_unit_str!("add t2, t3, t4", "add t2, t3, t4", 1000));
            asm!(unroll_unit_str!("add t2, t3, t4", "add t2, t3, t4", 1000));
            asm!(unroll_unit_str!(".insn 0x29de13b3", "add t2, t3, t4", 1000));
            asm!(unroll_unit_str!("add t2, t3, t4", "add t2, t3, t4", 1000));
            asm!(unroll_unit_str!("add t2, t3, t4", "add t2, t3, t4", 1000));
            asm!(unroll_unit_str!("add t2, t3, t4", "add t2, t3, t4", 1000));
            asm!(unroll_unit_str!("add t2, t3, t4", "add t2, t3, t4", 1000));
            asm!(unroll_unit_str!("add t2, t3, t4", "add t2, t3, t4", 1000));
        }
        for _ in 0..99 {
            execute_10_units!("add t2, t3, t4", "add t2, t3, t4", 1000);
        }
        let end = get_current_time();
        sum_001pct += end.saturating_sub(start);
    }
    unsafe {
        asm!("end_001pct:");
    }
    show_ratio_average("0.01%", sum_001pct);

    // --- 0% Ratio ---
    let mut sum_000pct: u64 = 0;
    unsafe {
        asm!("begin_000pct:");
    }
    for _ in 0..10 {
        reset_counter();
        let start = get_current_time();
        // 1000 * 10 + 1000 * 10 * 99 = 1000000
        execute_10_units!("add t2, t3, t4", "add t2, t3, t4", 1000);
        for _ in 0..99 {
            execute_10_units!("add t2, t3, t4", "add t2, t3, t4", 1000);
        }
        let end = get_current_time();
        sum_000pct += end.saturating_sub(start);
    }
    unsafe {
        asm!("end_000pct:");
    }
    show_ratio_average("0%", sum_000pct);

    // handle in M-mode
    let (err, _) = sbi::deleg_illegal_insn(false);
    if err != 0 {
        panic!("undefined ecall");
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Initialize the timer.
    init_timer();

    println!("hikami emulation overhead evaluation");
    println!("current time: {}", get_current_time());
    println!("current time: {}", get_current_time());
    println!("current time: {}", get_current_time());
    println!("current time: {}", get_current_time());
    println!("current time: {}", get_current_time());

    // rdcycle();
    time(false);
    time(true);
    time(false);
    time(true);

    mixed_ratio_benchmark();

    // End of the program.
    println!("End...");
    loop {}
}

struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            sbi::console_putchar(byte as usize);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print_fmt(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn print_fmt(args: core::fmt::Arguments) {
    use core::fmt::Write;
    let mut writer = Writer;
    writer.write_fmt(args).unwrap();
}
