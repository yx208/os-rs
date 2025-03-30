// 禁用标准库
#![no_std]
// 禁用 Rust 层级的入口点
#![no_main]

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

/// 程序 panic 时调用
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// 编译时不重整函数名
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }

    // 因为编译器会寻找一个名为 `_start` 的函数，所以这个函数就是入口点
    // 默认命名为 `_start`
    loop {}
}
