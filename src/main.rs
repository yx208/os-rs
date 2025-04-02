// 禁用标准库
#![no_std]
// 禁用 Rust 层级的入口点
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

static HELLO: &[u8] = b"Hello World!";

/// 程序 panic 时调用
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

/// 编译时不重整函数名
#[no_mangle]
pub extern "C" fn _start() -> ! {
    panic!("Some panic message");
    loop {}
}
