OUTPUT_ARCH( "riscv64" )
ENTRY( _start )

_stack_size = 0x400000;

SECTIONS
{
    . = 0x84000000;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }
    .data : {
        *(.data .data.*)
    }
    .bss : {
        *(.bss .bss.*)
        _bottom_stack = .;
        . += _stack_size;
        _top_stack = .;
    }
}
