#![no_std]
#![no_main]
#![feature(asm)]

// #[cfg(feature = "defmt")]
// use defmt_rtt as _; // global logger
// use panic_probe as _;

// use rustBoot_hal::stm::stm32f411::FlashWriterEraser;
// use rustBoot_update::update::{update_flash::FlashUpdater, UpdateInterface};
use shakti_riscv_hal::gpio::{GPIOInner, GPIO_OFFSET};
use shakti_riscv_hal::uart::{UartInner, UART_OFFSET};
use shakti_riscv_hal::spi::{SPIInner, SPI_OFFSET};
use riscv::asm;
use riscv_rt::{entry};
use core::arch::asm;
// use rustBoot::constants;

// use core::fmt::Write;
pub const BOOT_PARTITION_ADDRESS: usize = 0x80100000 ;

pub const SWAP_PARTITION_ADDRESS: usize = 0x80300000;

pub const UPDATE_PARTITION_ADDRESS: usize = 0x80200000;

struct UartInit{
    uart: UartInner,
}

#[entry]
fn main() -> ! {
    //let updater = FlashUpdater::new(FlashWriterEraser::new());
    //updater.rustboot_start()
    let mut uart = unsafe { UartInner::new(UART_OFFSET) };

    // ram address for firmware
    uart.write_uart_string("rustBoot in shakti initializing...\n ");
    let mut spi_flash = unsafe{SPIInner::new(SPI_OFFSET)};
    spi_flash.init();
    let dr5 = spi_flash.flash_device_id();
    // spi_flash.spi_tx_rx_start();
    
    // copy the code from external flash to ram after 1 MB of start address
    let mut bram_addr = BOOT_PARTITION_ADDRESS as *mut u32;
    let mut read_addr = 0x00bf0000;// external flash address starting address
    let mut read_val  = 0;

    while read_addr <= 0x00cf0000 {
        read_val = spi_flash.flash_read(read_addr);
        read_addr = read_addr + 4;
        unsafe{
            *bram_addr = read_val;
            bram_addr = bram_addr.offset(1);
        }
    }

    uart.write_uart_string("Value succeffully read...\n ");

    uart.write_uart_string("Control transferring to RAM");
    
    unsafe{
        // Execute the inline assembly code
        asm!(
            "fence.i",
            "li t6, 0x80100000", // Load immediate value into register t6
            "jr t6", // Jump to the address stored in register t6
            options(nomem, nostack, preserves_flags)
        );
    }
    
    
    loop{}
}

// unsafe fn jump_to_ram() {
//     uart.write_uart_string("Control transferred to RAM");
    
//     // Execute the inline assembly code
//     asm!(
//         "fence.i",
//         "li t6, 0x80000000", // Load immediate value into register t6
//         "jr t6", // Jump to the address stored in register t6
//         options(nomem, nostack, preserves_flags)
//     );
// }

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { riscv::asm::nop() };
    }
}

// should implement 
// #[panic_handler] // panicking behavior
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     loop {
//         cortex_m::asm::bkpt();
//     }
// }