#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

mod sbi;

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

// --- PLIC (Platform-Level Interrupt Controller) Constants ---
// NOTE: This is a common base address for the PLIC in many RISC-V platforms.
// You MUST verify and change this to the correct address for your specific hardware.
const PLIC_BASE: usize = 0x0c00_0000;
const PLIC_PRIORITY_OFFSET: usize = 0x0;
const PLIC_ENABLE_OFFSET: usize = 0x2000;
const PLIC_CONTEXT_OFFSET: usize = 0x200000;
const PLIC_THRESHOLD_OFFSET: usize = 0x0;
const PLIC_CLAIM_OFFSET: usize = 0x4;
// From the device tree: timer@0x51840000 has interrupt ID 0x159
const TIMER_INTERRUPT_ID: usize = 0x159;
// We are running in Supervisor mode on Hart 0. Context for S-mode is usually 1.
const PLIC_CONTEXT_S_MODE: usize = 1;

// Number of samples to measure.
const NUM_SAMPLES: usize = 100 + 1;

// Array to store the timestamps of the measurements.
static mut DELAYS: [u64; NUM_SAMPLES] = [0; NUM_SAMPLES];
// Counter for the current number of measurements.
static mut SAMPLE_COUNT: usize = 0;

// Number of cycles equivalent to 10ms (assuming a 24MHz clock for the hardware timer)
const TIMER_INTERVAL: u32 = 240_000;

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

/// Initializes the PLIC for the timer interrupt.
fn plic_init() {
    let plic_ptr = PLIC_BASE as *mut u32;
    unsafe {
        // Set priority for the timer interrupt source to 1 (lowest active priority).
        // Priority must be > 0.
        plic_ptr
            .byte_add(PLIC_PRIORITY_OFFSET + TIMER_INTERRUPT_ID * 4)
            .write_volatile(1);

        // Enable the timer interrupt for the Supervisor mode context.
        // The enable bits are organized in a bitmask.
        let enable_reg_offset =
            PLIC_ENABLE_OFFSET + PLIC_CONTEXT_S_MODE * 0x80 + (TIMER_INTERRUPT_ID / 32) * 4;
        let enable_bit = 1 << (TIMER_INTERRUPT_ID % 32);
        let current_enables = plic_ptr.byte_add(enable_reg_offset).read_volatile();
        plic_ptr
            .byte_add(enable_reg_offset)
            .write_volatile(current_enables | enable_bit);

        // Set the priority threshold for the Supervisor mode context to 0.
        // This allows any interrupt with a priority > 0 to be processed.
        plic_ptr
            .byte_add(PLIC_CONTEXT_OFFSET + PLIC_CONTEXT_S_MODE * 0x1000 + PLIC_THRESHOLD_OFFSET)
            .write_volatile(0);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Set the interrupt handler.
    unsafe {
        riscv::register::stvec::write(
            interrupt_handler as usize,
            riscv::register::stvec::TrapMode::Direct,
        );
    }

    // Initialize PLIC.
    plic_init();

    // Enable external interrupts in sie.
    unsafe {
        riscv::register::sie::set_sext();
    }

    // Enable all interrupts in sstatus.
    unsafe {
        riscv::register::sstatus::set_sie();
    }

    println!("External interrupt latency measurement start.");
    println!("Interval: {} cycles (10ms)", TIMER_INTERVAL);

    // Initialize and start the hardware timer.
    let timer_ptr = TIMER_BASE as *mut u32;
    unsafe {
        timer_ptr
            .byte_add(APBTMR_N_LOAD_COUNT)
            .write_volatile(TIMER_INTERVAL);
        let control_val = APBTMR_CONTROL_ENABLE | APBTMR_CONTROL_MODE_PERIODIC;
        timer_ptr
            .byte_add(APBTMR_N_CONTROL)
            .write_volatile(control_val);
    }

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

    // Disable external interrupts.
    unsafe {
        riscv::register::sie::clear_sext();
    }

    // Stop the timer.
    unsafe {
        let timer_ptr = TIMER_BASE as *mut u32;
        timer_ptr.byte_add(APBTMR_N_CONTROL).write_volatile(0);
    }

    // Print the results.
    print_results();

    println!("Measurement finished.");

    // End of the program.
    loop {}
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text")]
pub unsafe extern "C" fn interrupt_handler() {
    unsafe {
        asm!(
            // Save context
            "
        .align 4
        addi sp, sp, -256

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
        addi sp, sp, 256
        sret
        ",
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn handle_interrupt(scause: usize, drift_time: u64) {
    // Supervisor External Interrupt (scause = 0x8000000000000009)
    if scause == 0x8000_0000_0000_0009 {
        let plic_ptr = PLIC_BASE as *mut u32;
        let claim_addr = unsafe {
            plic_ptr
                .byte_add(PLIC_CONTEXT_OFFSET + PLIC_CONTEXT_S_MODE * 0x1000 + PLIC_CLAIM_OFFSET)
        };

        // Claim the interrupt from PLIC.
        let interrupt_id = unsafe { claim_addr.read_volatile() as usize };

        // Check if the interrupt is from our timer.
        if interrupt_id == TIMER_INTERRUPT_ID {
            // Record the current time.
            unsafe {
                if SAMPLE_COUNT < NUM_SAMPLES {
                    DELAYS[SAMPLE_COUNT] = TIMER_INTERVAL as u64 - drift_time;
                    SAMPLE_COUNT += 1;
                }
            }

            // Clear the timer's internal interrupt pending bit by reading the EOI register.
            let timer_ptr = TIMER_BASE as *mut u32;
            unsafe {
                let _ = timer_ptr.byte_add(APBTMRS_N_EOI).read_volatile();
            }
        }

        // Signal completion to PLIC.
        if interrupt_id != 0 {
            unsafe {
                claim_addr.write_volatile(interrupt_id as u32);
            }
        }
    }
}

/// Prints the measurement results.
fn print_results() {
    println!("\n--- Measurement Results ---");
    let mut delays = [0u64; NUM_SAMPLES - 1];
    let mut sum = 0;

    for i in 1..(NUM_SAMPLES - 1) {
        unsafe {
            sum += DELAYS[i];
        }
        println!("[{}]: {} cycles", i, unsafe { DELAYS[i] });
    }

    let avg = sum / (NUM_SAMPLES - 1 - 1) as u64;
    println!("-------------------------");
    println!("Average latency: {} cycles", avg);
    println!(
        "Average latency (vs expected): {} us",
        avg as i64 - 10_000_i64
    );
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
