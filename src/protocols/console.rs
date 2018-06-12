use ffi::{
    console::{
        EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL, 
        EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL, 
        EFI_KEY_DATA,
        EFI_SHIFT_STATE_VALID,
        EFI_LEFT_CONTROL_PRESSED,
        EFI_RIGHT_CONTROL_PRESSED,
    }, 
    IsSuccess, 
    UINTN,
};
use core::cmp;
use io::{self, Cursor};
use EfiError;
use ::Result;
use system_table;
use alloc::{Vec, String, str};

// TODO: This whole module has gotten ugly. Needs cleanup.
// TODO: Should we replace Console with two structs, StdIn and StdOut, corresponding to input and output? This is more in line with Rust stdlib.
pub struct Console {
    pub input: *const EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL,
    pub output: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    utf8_buf: io::Cursor<Vec<u8>>
}

const LF: u16 = 10;
const CR: u16 = 13;
const BS: u16 = 8;

impl Console {
    pub fn new(input: *const EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL, output: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL) -> Self {
        Self { input, output, utf8_buf: Cursor::new(Vec::new()) }
    }

    fn write_to_efi(&self, buf: &[u16]) -> Result<()> {
        unsafe {
            let (ptr, _) = to_ptr(buf);
            ret_on_err!(((*(*self).output).OutputString)(self.output, ptr));
            Ok(())
        }
    }

    // TODO: code in this function is super ugly and prone to bugs. Clean it up.
    fn read_from_efi(&self, buf: &mut [u16]) -> Result<usize> {
        let mut bytes_read = 0;

        let input = (*self).input as *mut EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL;
        let mut evt_index: UINTN = 0;
        let mut key_data = EFI_KEY_DATA::default();
        let mut evt_list = unsafe { [(*input).WaitForKeyEx; 1] };

        while bytes_read < buf.len() {
            // TODO: For some reason we can't use ret_on_err here. Why?
            let status = unsafe { ((*system_table().BootServices).WaitForEvent)(evt_list.len(), evt_list.as_mut_ptr(), &mut evt_index) };
            if !IsSuccess(status) {
                return Err(status.into()); // TODO: Can we send some error text too with such errors
            }

            // TODO: For some reason we can't use ret_on_err here. Why?
            let status = unsafe { ((*input).ReadKeyStrokeEx)(input, &mut key_data) };
            if !IsSuccess(status) {
                return Err(status.into()); // TODO: Can we send some error text too with such errors
            }

            fn is_ctr_z(key_data: &EFI_KEY_DATA) -> bool {
                (key_data.Key.UnicodeChar == 'z' as u16 || key_data.Key.UnicodeChar == 'Z' as u16) && 
                (key_data.KeyState.KeyShiftState & EFI_SHIFT_STATE_VALID) != 0  &&
                ((key_data.KeyState.KeyShiftState & EFI_LEFT_CONTROL_PRESSED) != 0 || (key_data.KeyState.KeyShiftState & EFI_RIGHT_CONTROL_PRESSED) != 0) 
            }

            if key_data.Key.UnicodeChar != 0 { // != 0 means it's a printable unicode char
                if key_data.Key.UnicodeChar == CR { // Safe to check for CR only without waiting for LF because in my experience UEFI only ever inserts CR when you press the Enter key
                    key_data.Key.UnicodeChar = LF; // Always translate CR's to LF's to normalize line endings.
                }

                match key_data.Key.UnicodeChar {
                    BS => {
                        if bytes_read > 0 {
                            bytes_read -= 1;
                            self.write_to_efi(&[BS, 0])?; // 0 is for null termination
                        }
                    },
                    c => {
                        if is_ctr_z(&key_data) {
                            break;
                        } else {
                            buf[bytes_read] = c;
                            bytes_read += 1;

                            if c == LF {
                                self.write_to_efi(&[CR, LF, 0])?; // Must echo both CR and LF because other wise it fucks up the cursor position.
                                break;
                            } else {
                                self.write_to_efi(&[c, 0])?;
                            }
                        }
                    }
                };
            } else {
                // TODO: handle scan codes here.
            }

            // TODO: should also support ctrl+z as a terminating sequence?
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

        // Convert to UTF16, normalizing all LF's to CRLF's (if present)
        // because UEFI console doesn't automatically perform carriage upon seeing LF's
        let utf16_iter = utf8_buf.encode_utf16();
        let mut expected_utf16_buf_size = utf16_iter.size_hint().1.unwrap_or(utf8_buf.len()); // Guessing the capacity of utf16 buffer
        expected_utf16_buf_size = (expected_utf16_buf_size as f32 * 1.05) as usize; // Adding 5% extra in case we have to normalize line endings
        let mut utf16_buf = Vec::with_capacity(expected_utf16_buf_size);

        let mut last_c = 0_u16;
        for (i, c) in utf16_iter.enumerate() {
            if i >= 1 && c == LF && last_c != CR { // Normalizing LF's
                utf16_buf.push(CR);
            }
            utf16_buf.push(c);
            last_c = c;
        }

        utf16_buf.push(0); // Appending the null terminator

        self.write_to_efi(&utf16_buf)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to write to EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL"))?; // TODO: Don't swallaow EFI status like this. Error handling in this whole crate needs fixing

        Ok(utf8_buf.len())
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
