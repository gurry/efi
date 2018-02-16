use ffi::{console::{EFI_SIMPLE_TEXT_INPUT_PROTOCOL, EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL}, EFI_STATUS, IsError};
use core::fmt;

pub struct Console {
    pub input: *const EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
    pub output: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
}

// TODO: write! macros works fine but writeln! doesn't because it inserts into \n characters not the \r\n char pairs. Fix this somehow.
const WRITE_BUFSIZE: usize = 512;
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut buf = [0u16; WRITE_BUFSIZE];
        let mut i = 0;

        let mut chars = s.chars();

        loop {
            let mut chunk = chars.by_ref().take(WRITE_BUFSIZE - 1).peekable();

            if chunk.peek() == None {
                // TODO: Currently swallowing all warnings in this method. Should we turn them into errors?
                return Ok(());
            }

            for c in chunk {
                buf[i] = c as u16;
                i += 1;
            }

            buf[i] = 0;
            let status = self.write_to_efi(&buf);
            if IsError(status) {
                return Err(fmt::Error);
            }
            i = 0;
        }
    }
}

impl Console {
    fn write_to_efi(&self, buf: &[u16]) -> EFI_STATUS {
        unsafe {
            let (ptr, _) = to_ptr(buf);
            ((*(*self).output).OutputString)(self.output, ptr)
        }
    }
}

fn to_ptr<T>(slice: &[T]) -> (*const T, usize) {
    unsafe {
        transmute(slice)
    }
}

extern "rust-intrinsic" {
    fn transmute<T,U>(val: T) -> U;
}
