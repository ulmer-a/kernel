OUTPUT_ARCH("i686")
ENTRY(_multiboot_entry)

PROVIDE(__stack_top = 0x80000);

SECTIONS
{
    . = 0x100000;

    __kernel_start = .;

    .multiboot : {
        __text_start = .;
        *(.multiboot)
        KEEP(*(.multiboot))
    }

    .text : {
        *(.text .text.*)

        . = ALIGN(4096);
        __text_end = .;
    }

    .rodata : {
        . = ALIGN(4096);
        __rodata_start = .;

        *(.rodata*)

        . = ALIGN(4096);
        __rodata_end = .;
    }

    .data : {
        . = ALIGN(4096);
        __rw_data_start = .;

        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
        
        __rw_data_end = .;

        . = ALIGN(4096);
    }

    __kernel_end = .;
}
