#![no_std]
#![no_main]
#![feature(asm)]

use riscv::{asm::delay, delay};
// use cortex_m_rt::entry;
use crate::FlashInterface;
use riscv_rt::entry;
use shakti_riscv_hal::spi::{SPIInner, SPI_OFFSET};
use shakti_riscv_hal::uart::{UartInner, UART_OFFSET};


pub struct FlashWriterEraser {
    pub spi: SPIInner,
}

impl FlashWriterEraser {
    pub fn new() -> Self {
        let spi_inner = unsafe { SPIInner::new(SPI_OFFSET) };
        spi.init();
        FlashWriterEraser { spi: spi_inner }
    }
}

impl FlashInterface for FlashWriterEraser {
    /// This method is to write data on flash
    ///
    /// Method arguments:
    /// -   address: It holds the address of flash where data has to be written
    /// -   data: u8 pointer holding the holding data.
    /// -   len :  number of bytes
    ///
    /// Returns:
    /// -  NONE
    fn hal_flash_write(&self, address: usize, data: *const u8, len: usize) {
        let address = address as u32;
        let len = len as u32;
        let mut idx = 0u32;
        let mut src = data as *mut u32;
        let mut dst = address as *mut u32;
        //Unlock the FLASH
        while idx < len {
            let data_ptr = (data as *const u32) as u32;
            //checking if the len is more than 4 bytes to compute a 4 byte write on flash
            if (len - idx > 3) {
                // Enable FLASH Page writes
                let b = spi.flash_read(dst);

                spi.flash_write_enable();
                // spi.flash_erase(0x00b0_0000);
                spi.flash_status_register_read();

                let z = spi.flash_write(dst, src);
                src = ((src as u32) + 4) as *mut u32; // increment pointer by 4
                dst = ((dst as u32) + 4) as *mut u32; // increment pointer by 4
                idx += 4;
            } else {
                // else do a single byte write i.e. 1-byte write
                let mut val = 0u32;
                let val_bytes = ((&mut val) as *mut u32) as *mut u8;
                let offset = (address + idx) - (((address + idx) >> 2) << 2); // offset from nearest word aligned address
                dst = ((dst as u32) - offset) as *mut u32; // subtract offset from dst addr
                unsafe {
                    val = *dst; // assign current val at dst to val
                                // store data byte at idx to `val`. `val_bytes` is a byte-pointer to val.
                    *val_bytes.add(offset as usize) = *data.add(idx as usize);
                }
                let b = spi.flash_read(dst);

                spi.flash_write_enable();
                // spi.flash_erase(0x00b0_0000);
                spi.flash_status_register_read();

                let z = spi.flash_write(dst, val);

                src = ((src as u32) + 1) as *mut u32; // increment pointer by 1
                dst = ((dst as u32) + 1) as *mut u32; // increment pointer by 1
                idx += 1;
            }
        }
        //Lock the FLASH
        self.hal_flash_lock();
    }

    /// This method is used to erase data on flash
    ///
    /// In STM32F411 only sector erase is available. whatever be the length of bytes we pass to this function will erase
    /// the whole sector, whichever the sector the address belong to.
    ///
    /// Method arguments:
    /// -   addr: Address where data has to be erased
    /// -   len :  number of bytes to be erased
    ///
    /// Returns:
    /// -  NONE

    fn hal_flash_erase(&self, addr: usize, len: usize) {
        self.spi.flash_read(0x00B0_0000);

        self.spi.flash_write_enable();
        self.spi.flash_erase(0x00b0_0000);
    }
    /// This method is used to lock the flash
    ///
    /// Once the flash is locked no operation on flash can be perfomed.
    /// Method arguments:
    /// -   NONE
    /// Returns:
    /// -  NONE
    fn hal_flash_lock(&self) {}
    /// This method is used to unlock the flash
    ///
    /// Flash has to be unlocked to do any operation on it.
    /// Method arguments:
    /// -   NONE
    /// Returns:
    /// -  NONE
    fn hal_flash_unlock(&self) {}
    fn hal_init() {}
}
pub fn preboot() {}

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { riscv::asm::nop() };
    }
}
