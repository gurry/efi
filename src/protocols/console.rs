use ffi::{console::{EFI_SIMPLE_TEXT_INPUT_PROTOCOL, EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL}, IsSuccess};
use core::fmt;
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
        self.write_u16(&mut s.chars().map(|c| c as u16)).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

impl Console {
    fn write_u16<I: Iterator<Item=u16>>(&mut self, iter: &mut I) -> Result<()> {
        let mut buf = [0u16; WRITE_BUFSIZE];
        let mut i = 0;

        loop {
            let mut chunk = iter.by_ref().take(WRITE_BUFSIZE - 1).peekable();

            if chunk.peek() == None {
                // TODO: Currently swallowing all warnings in this method. Should we turn them into errors?
                return Ok(());
            }

            for c in chunk {
                buf[i] = c;
                i += 1;
            }

            buf[i] = 0;
            self.write_to_efi(&buf)?;
            i = 0;
        }
    }

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
        self.write_u16(&mut buf.iter().map(|i| *i as u16))
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to write to EFI_SIMPLE_OUTPUT_PROTOCOL"))?; // TODO: Don't swallaow EFI status like this. Error handling in this whole crate needs fixing

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
