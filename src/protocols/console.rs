use ffi::{console::{EFI_SIMPLE_TEXT_INPUT_PROTOCOL, EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL}, IsError, IsSuccess};
use core::{fmt, mem};
use io;
use EfiError;
use ::Result;

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
            self.write_to_efi(&buf).map_err(|_| fmt::Error)?;
            i = 0;
        }
    }
}

impl Console {
    fn write_to_efi(&self, buf: &[u16]) -> Result<()> {
        unsafe {
            let (ptr, _) = to_ptr(buf);
            ret_on_err!(((*(*self).output).OutputString)(self.output, ptr));
            Ok(())
        }
    }
}

impl io::Write for Console {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() % 2 != 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Provided buffer has odd length. Cannot interpret as a UCS-2 buffer."));
        }

        let buf: &[u16] = unsafe { mem::transmute(buf) };
        self.write_to_efi(buf).map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to write to EFI_SIMPLE_OUTPUT_PROTOCOL"))?; // TODO: Don't swallaow EFI status like this. Error handling in this whole crate needs fixing
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(()) // Do nothing. UEFI SIMPLE_TEXT_OUTPUT protocol does not support flushing
    } 
}

fn to_ptr<T>(slice: &[T]) -> (*const T, usize) {
    unsafe {
        transmute(slice)
    }
}

// TODO is this really needed? Doesn't core expose mem::transmute?
extern "rust-intrinsic" {
    fn transmute<T,U>(val: T) -> U;
}
