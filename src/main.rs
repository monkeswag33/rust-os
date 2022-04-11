#![no_std]
#![no_main]
use core::panic::PanicInfo;
mod vga_buffer;
use vga_buffer::WRITER;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Print Hello World to screen
    WRITER.lock().write_string("Hello World");
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
