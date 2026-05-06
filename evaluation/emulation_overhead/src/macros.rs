// Basic repeating units for string concatenation
#[rustfmt::skip]
macro_rules! base_1 { ($b:expr) => { concat!($b, "\n") }; }
#[rustfmt::skip]
macro_rules! base_10 { ($b:expr) => { concat!(base_1!($b),base_1!($b),base_1!($b),base_1!($b),base_1!($b),base_1!($b),base_1!($b),base_1!($b),base_1!($b),base_1!($b)) }; }
#[rustfmt::skip]
macro_rules! base_100 { ($b:expr) => { concat!(base_10!($b),base_10!($b),base_10!($b),base_10!($b),base_10!($b),base_10!($b),base_10!($b),base_10!($b),base_10!($b),base_10!($b)) }; }
#[rustfmt::skip]
macro_rules! base_1000 { ($b:expr) => { concat!(base_100!($b),base_100!($b),base_100!($b),base_100!($b),base_100!($b),base_100!($b),base_100!($b),base_100!($b),base_100!($b),base_100!($b)) }; }

// Generates a string of a specific number of instructions including one target in the middle
#[rustfmt::skip]
macro_rules! unroll_unit_str {
    ($target:literal, $base:literal, 100) => {
        concat!(
            base_10!($base), base_10!($base), base_10!($base), base_10!($base),
            base_1!($base), base_1!($base), base_1!($base), base_1!($base), base_1!($base),
            base_1!($base), base_1!($base), base_1!($base), base_1!($base),
            $target, "\n",
            base_10!($base), base_10!($base), base_10!($base), base_10!($base), base_10!($base)
        )
    };
    ($target:literal, $base:literal, 1000) => {
        concat!(
            base_100!($base), base_100!($base), base_100!($base), base_100!($base),
            base_10!($base), base_10!($base), base_10!($base), base_10!($base), base_10!($base),
            base_10!($base), base_10!($base), base_10!($base), base_10!($base),
            base_1!($base), base_1!($base), base_1!($base), base_1!($base), base_1!($base),
            base_1!($base), base_1!($base), base_1!($base), base_1!($base),
            $target, "\n",
            base_100!($base), base_100!($base), base_100!($base), base_100!($base), base_100!($base)
        )
    };
    ($target:literal, $base:literal, 10000) => {
        concat!(
            // 4999
            base_1000!($base), base_1000!($base), base_1000!($base), base_1000!($base),
            base_100!($base), base_100!($base), base_100!($base), base_100!($base), base_100!($base),
            base_100!($base), base_100!($base), base_100!($base), base_100!($base),
            base_10!($base), base_10!($base), base_10!($base), base_10!($base), base_10!($base),
            base_10!($base), base_10!($base), base_10!($base), base_10!($base),
            base_1!($base), base_1!($base), base_1!($base), base_1!($base), base_1!($base),
            base_1!($base), base_1!($base), base_1!($base), base_1!($base),
            $target, "\n",
            // 5000
            base_1000!($base), base_1000!($base), base_1000!($base), base_1000!($base), base_1000!($base)
        )
    };
}

#[rustfmt::skip]
macro_rules! execute_10_units {
    ($target:literal, $base:literal, $ratio:tt) => {
        unsafe {
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
            asm!(unroll_unit_str!($target, $base, $ratio), options(nomem, nostack, preserves_flags));
        }
    };
}
