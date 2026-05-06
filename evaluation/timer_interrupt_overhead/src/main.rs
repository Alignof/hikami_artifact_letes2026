// file: src/main.rs

#![no_std]
#![no_main]

use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;

mod sbi;

// Number of samples to measure.
const NUM_SAMPLES: usize = 100;

// --- Timer Hardware Constants ---
// Base address of the timer from the device tree (timer@0x51840000)
const TIMER_BASE: usize = 0x5184_0000;
// Timer register offsets from the driver code
const APBTMR_N_LOAD_COUNT: usize = 0x00;
const APBTMR_N_CURRENT_VALUE: usize = 0x04;
const APBTMR_N_CONTROL: usize = 0x08;
const APBTMRS_N_EOI: usize = 0x0c;
// Timer control register bits
const APBTMR_CONTROL_ENABLE: u32 = 1 << 0;
const APBTMR_CONTROL_MODE_PERIODIC: u32 = 1 << 1;
const APBTMR_CONTROL_INTERRUPT: u32 = 1 << 2;

// Array to store the timestamps of the measurements.
static mut DELAYS: [u64; NUM_SAMPLES] = [0; NUM_SAMPLES];
// Array to store the timestamps of the measurements.
static mut TIMESTAMPS: [u32; NUM_SAMPLES] = [0; NUM_SAMPLES];
// Counter for the current number of measurements.
static mut SAMPLE_COUNT: usize = 0;

// Number of cycles equivalent to 10ms (the clock frequency of the QEMU virt machine is 10MHz).
const TIMER_INTERVAL: u32 = 100_000;

unsafe extern "C" {
    /// Stack top (defined in `linker.ld`)
    pub static _top_stack: u8;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop {
        riscv::asm::wfi();
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        // Initialize the stack pointer.
        "la sp, {stack_end}",
        // Jump to the main function.
        "j main",
        stack_end = sym _top_stack,
    );
}

#[unsafe(no_mangle)]
extern "C" fn main() -> ! {
    // Set the interrupt handler.
    unsafe {
        asm!("csrw stvec, {}", in(reg) interrupt_handler as usize);
    }

    // Enable timer interrupts.
    unsafe {
        asm!("csrs sie, {}", in(reg) 1 << 5); // Set the STIE bit.
    }

    // Enable all interrupts.
    unsafe {
        asm!("csrs sstatus, {}", in(reg) 1 << 1); // Set the SIE bit.
    }

    // Initialize and start the hardware timer.
    let timer_ptr = TIMER_BASE as *mut u32;
    unsafe {
        timer_ptr
            .byte_add(APBTMR_N_LOAD_COUNT)
            .write_volatile(0xffff_ffff);
        // let control_val = APBTMR_CONTROL_ENABLE | APBTMR_CONTROL_MODE_PERIODIC;
        let control_val = APBTMR_CONTROL_ENABLE | APBTMR_CONTROL_INTERRUPT;
        timer_ptr
            .byte_add(APBTMR_N_CONTROL)
            .write_volatile(control_val);
    }

    println!("Timer interrupt latency measurement start.");
    println!("Interval: {} cycles (10ms)", TIMER_INTERVAL);

    // Set the first timer interrupt.
    let target_time = riscv::register::time::read64() + TIMER_INTERVAL as u64;
    sbi::set_timer(target_time);

    // Wait until the measurements are complete.
    loop {
        if unsafe { SAMPLE_COUNT } >= NUM_SAMPLES {
            break;
        }
        // Wait for an interrupt.
        unsafe {
            asm!("wfi");
        }
    }

    // Enable timer interrupts.
    unsafe {
        asm!("csrc sie, {}", in(reg) 1 << 5); // Clear the STIE bit.
    }

    // Print the results.
    print_results();

    println!("Measurement finished.");

    // End of the program.
    loop {}
}

#[unsafe(naked)]
#[unsafe(link_section = ".text")]
unsafe extern "C" fn interrupt_handler() {
    naked_asm!(
        // Save the context.
        "
        .align 4
        addi sp, sp, -{HS_CONTEXT_SIZE}

        sd a1, 11*8(sp)
        la a1, 0x51840004
        lw a1, 0(a1)

        sd ra, 1*8(sp)
        sd gp, 3*8(sp)
        sd tp, 4*8(sp)
        sd t0, 5*8(sp)
        sd t1, 6*8(sp)
        sd t2, 7*8(sp)
        sd s0, 8*8(sp)
        sd s1, 9*8(sp)
        sd a0, 10*8(sp)
        // sd a1, 11*8(sp)
        sd a2, 12*8(sp)
        sd a3, 13*8(sp)
        sd a4, 14*8(sp)
        sd a5, 15*8(sp)
        sd a6, 16*8(sp)
        sd a7, 17*8(sp)
        sd s2, 18*8(sp)
        sd s3, 19*8(sp)
        sd s4, 20*8(sp)
        sd s5, 21*8(sp)
        sd s6, 22*8(sp)
        sd s7, 23*8(sp)
        sd s8, 24*8(sp)
        sd s9, 25*8(sp)
        sd s10, 26*8(sp)
        sd s11, 27*8(sp)
        sd t3, 28*8(sp)
        sd t4, 29*8(sp)
        sd t5, 30*8(sp)
        sd t6, 31*8(sp)

        // Read scause
        csrr a0, scause 
        // Call handler
        call handle_interrupt

        // Restore context
        ld ra, 1*8(sp)
        ld gp, 3*8(sp)
        ld tp, 4*8(sp)
        ld t0, 5*8(sp)
        ld t1, 6*8(sp)
        ld t2, 7*8(sp)
        ld s0, 8*8(sp)
        ld s1, 9*8(sp)
        ld a0, 10*8(sp)
        ld a1, 11*8(sp)
        ld a2, 12*8(sp)
        ld a3, 13*8(sp)
        ld a4, 14*8(sp)
        ld a5, 15*8(sp)
        ld a6, 16*8(sp)
        ld a7, 17*8(sp)
        ld s2, 18*8(sp)
        ld s3, 19*8(sp)
        ld s4, 20*8(sp)
        ld s5, 21*8(sp)
        ld s6, 22*8(sp)
        ld s7, 23*8(sp)
        ld s8, 24*8(sp)
        ld s9, 25*8(sp)
        ld s10, 26*8(sp)
        ld s11, 27*8(sp)
        ld t3, 28*8(sp)
        ld t4, 29*8(sp)
        ld t5, 30*8(sp)
        ld t6, 31*8(sp)
        addi sp, sp, {HS_CONTEXT_SIZE}

        sret
        ",
            HS_CONTEXT_SIZE = const 256,
    );
}

#[unsafe(no_mangle)]
extern "C" fn handle_interrupt(scause: usize, interrupt_time: u32) {
    let current_time = riscv::register::time::read64();

    // Supervisor Timer Interrupt (scause = 0x8000000000000005)
    if scause == 0x8000_0000_0000_0005 {
        unsafe {
            if SAMPLE_COUNT < NUM_SAMPLES {
                TIMESTAMPS[SAMPLE_COUNT] = interrupt_time;
                SAMPLE_COUNT += 1;
            }
        }

        // Set the next timer interrupt.
        let target_time = current_time + TIMER_INTERVAL as u64;
        sbi::set_timer(target_time);

        // Clear the timer interrupt pending bit in the sip register.
        unsafe {
            asm!("csrc sip, {}", in(reg) 1 << 5);
        }
    }
}

/// Prints the measurement results.
fn print_results() {
    println!("\n--- Measurement Results ---");
    let mut delays = [0u64; NUM_SAMPLES - 1];
    let mut sum: u64 = 0;
    for i in 1..(NUM_SAMPLES - 1) {
        let delay = unsafe { TIMESTAMPS[i] - TIMESTAMPS[i + 1] };
        sum += delay as u64;
        println!("[{}]: time stamp {} cycles", i, unsafe { TIMESTAMPS[i] });
        println!("[{}]: delay {} cycles", i, delay);
    }

    let avg = sum / (NUM_SAMPLES - 1 - 1) as u64;
    println!("-------------------------");
    println!("Average latency: {} cycles", avg);
    println!(
        "Average latency (vs expected): {} us",
        avg as i64 - 240_000_i64
    );
}

// --- Simple implementation for output ---
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
