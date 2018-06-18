use protocols::console::stdout;
use alloc::fmt;
use io::Write;

#[macro_export]
macro_rules! format {
    ($($arg:tt)*) => (alloc::fmt::format(format_args!($($arg)*)))
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::fmt::print_args(format_args!($($arg)*)));
}

// TODO: Call to stdout() creates a new StdOut obj everytime. Remove this extravagance.
pub fn print_args(args: fmt::Arguments) {
    return stdout().write_fmt(args).expect("Failed to write to stdout")
}

