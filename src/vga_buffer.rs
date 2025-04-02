//！ VGA 字符单元布局
//！ Bit(s)	Value
//！ 0-7	    ASCII code point
//！ 8-11	Foreground color
//！ 12-14	Background color
//！ 15	    Blink
use core::fmt;
use lazy_static::lazy_static;
use volatile::Volatile;
use spin::Mutex;

/// 默认情况下，Rust 编译器可以自由选择枚举的内存布局和大小，但使用 repr 属性可以明确指定
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// "repr(transparent)" 让包装类型在内存中的表示与被包装的类型完全一致
/// 使 ColorCode 跟 u8 内存布局相同
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// 使用一个 u8 储存前景背景色
    /// 如果背景是白色(0000 1111)，前景是蓝色(0000 0001)
    /// (0000 1111) << 4 = (1111 0000)
    /// (1111 0000) | 蓝色(0000 0001) = (1111 0001)
    fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

/// "repr(C)" 指定结构体或枚举在内存中的布局方式应当遵循 C 语言的规则
/// 意味着
/// 1. 结构体字段按照声明顺序排列
/// 2. 遵循 C 语言的对齐规则
/// 3. 不进行字段重排优化
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    // 默认背景色
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// 让这个 Writer 类型将字符写入屏幕的最后一行，并在一行写满或接收到换行符 \n 的时候，将所有的字符向上位移一行
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// 从第 1 行开始，省略了对第 0 行的枚举过程——因为这一行应该被移出屏幕，即它将被下一行的字符覆写
    pub fn new_line(&mut self) {
        // 将最后一行的字符往上提
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        // VGA 字符缓冲区只支持 ASCII 码字节和代码页 437 定义的字节
        for byte in s.bytes() {
            match byte {
                // 可以是能打印的 ASCII 码字节，也可以是换行符
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // 不包含在上述范围之内的字节
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// 问题 1
/// 一般的变量在运行时初始化，而静态变量在编译时初始化
/// Rust 编译器规定了一个称为常量求值器（const evaluator）的组件，它应该在编译时处理这样的初始化工作
/// lazy_static 宏可以定义一个延迟初始化（lazily initialized）的静态变量
/// 这个变量的值将在第一次使用时计算，而非在编译时计算
///
/// 问题 2
/// 所有与写入数据相关的方法都需要实例的可变引用 "&mut self"，但 WRITER 是 不可变变量
/// 使用自旋锁，提供内部可变性
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
