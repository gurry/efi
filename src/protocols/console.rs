use ffi::{
    console::{EFI_SIMPLE_TEXT_INPUT_PROTOCOL, EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL, EFI_INPUT_KEY}, 
    IsSuccess, 
    UINTN,
    EFI_EVENT,
};
use core::{fmt, cmp};
use io::{self, Cursor};
use EfiError;
use ::Result;
use system_table;
use alloc::{Vec, String, str};

pub struct Console {
    pub input: *const EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
    pub output: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    utf8_buf: io::Cursor<Vec<u8>>
}

// TODO: write! macros works fine but writeln! doesn't because it inserts into \n characters not the \r\n char pairs. Fix this somehow.
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use io::Write;

        let buf = s.as_bytes();
        let mut total_written = 0;
        while total_written < buf.len() {
            let written = self.write(&buf[total_written..]).map_err(|_| fmt::Error)?; // TODO: Swalling upstream errors. Do not, if possible.
            if written == 0 {
                return Err(fmt::Error)
            }

            total_written += written;
        }
        Ok(())
    }
}

impl Console {
    pub fn new(input: *const EFI_SIMPLE_TEXT_INPUT_PROTOCOL, output: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL) -> Self {
        Self { input, output, utf8_buf: Cursor::new(Vec::new()) }
    }

    fn write_to_efi(&self, buf: &[u16]) -> Result<()> {
        unsafe {
            let (ptr, _) = to_ptr(buf);
            ret_on_err!(((*(*self).output).OutputString)(self.output, ptr));
            Ok(())
        }
    }

    fn read_from_efi(&self, buf: &mut [u16]) -> Result<usize> {
        let mut bytes_read = 0;

        let input = (*self).input as *mut EFI_SIMPLE_TEXT_INPUT_PROTOCOL;
        let mut evt_index: UINTN = 0;
        let mut key = EFI_INPUT_KEY::default();

        while bytes_read < buf.len() {
            // TODO: For some reason we can't use ret_on_err here. Why?
            let status = unsafe { ((*system_table().BootServices).WaitForEvent)(1, (*input).WaitForKey as *const EFI_EVENT, &mut evt_index) };
            if !IsSuccess(status) {
                return Err(status.into()); // TODO: Can we send some error text too with such errors
            }

            // TODO: For some reason we can't use ret_on_err here. Why?
            let status = unsafe { ((*input).ReadKeyStroke)(input, &mut key) };
            if !IsSuccess(status) {
                return Err(status.into()); // TODO: Can we send some error text too with such errors
            }

            if key.UnicodeChar >= ' ' as u16 { // Only if it's a non-ctrl char
                buf[bytes_read] = key.UnicodeChar;
                bytes_read += 1;
                // TODO: do we need to echo the char back to Output?
            } else if key.UnicodeChar == '\n' as u16 { // TODO: should also support ctrl+z as a terminating sequence?
                break;
            } 
        }

        Ok(bytes_read)
    }
}

impl io::Write for Console {
    /// Writes given UTF8 buffer to the console.
    /// UEFI console natively only supports UCS-2.
    /// Therefore any code-points above the BMP will show up garbled.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        const WRITE_BUFSIZE: usize = 8192;
        let bytes_to_write = cmp::min(buf.len(), WRITE_BUFSIZE);
        let utf8_buf = match str::from_utf8(&buf[..bytes_to_write]) {
            Ok(str_) => str_,
            Err(ref e) if e.valid_up_to() == 0 => return Err(invalid_encoding()),
            Err(e) => str::from_utf8(&buf[..e.valid_up_to()]).unwrap(), // At least write those that are valid
        };

        let utf16_buf = utf8_buf.encode_utf16().collect::<Vec<u16>>();
        self.write_to_efi(&utf16_buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to write to EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL"))?; // TODO: Don't swallaow EFI status like this. Error handling in this whole crate needs fixing

        Ok(utf16_buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(()) // Do nothing. UEFI SIMPLE_TEXT_OUTPUT protocol does not support flushing
    } 
}


impl io::Read for Console {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Read more if the buffer is empty
        if self.utf8_buf.position() as usize == self.utf8_buf.get_ref().len() {
            let mut utf16_buf = vec![0u16; 0x1000];
            let bytes_read = self.read_from_efi(&mut utf16_buf).map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to read from EFI_SIMPLE_TEXT_INPUT_PROTOCOL"))?; // TODO: again swallowing incoming efi status
            utf16_buf.truncate(bytes_read as usize);
            // FIXME: what to do about this data that has already been read?
            let data = match String::from_utf16(&utf16_buf) {
                Ok(utf8_buf) => utf8_buf.into_bytes(),
                Err(..) => return Err(invalid_encoding()),
            };

            self.utf8_buf = Cursor::new(data);
        }

        // MemReader shouldn't error here since we just filled it
        self.utf8_buf.read(buf)
    }
}

fn invalid_encoding() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "text was not valid unicode")
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
