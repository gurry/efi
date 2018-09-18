use ffi::{
    console::{
        EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL, 
        EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
        EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL, 
        EFI_KEY_DATA,
        EFI_INPUT_KEY,
        EFI_SHIFT_STATE_VALID,
        EFI_LEFT_CONTROL_PRESSED,
        EFI_RIGHT_CONTROL_PRESSED,
        EFI_BLACK,
        EFI_BLUE,
        EFI_GREEN,
        EFI_CYAN,
        EFI_RED,
        EFI_MAGENTA,
        EFI_BROWN,
        EFI_LIGHTGRAY,
        EFI_DARKGRAY,
        EFI_LIGHTBLUE,
        EFI_LIGHTGREEN,
        EFI_LIGHTCYAN,
        EFI_LIGHTRED,
        EFI_LIGHTMAGENTA,
        EFI_YELLOW,
        EFI_WHITE,
        EFI_BACKGROUND_BLACK,
        EFI_BACKGROUND_BLUE,
        EFI_BACKGROUND_GREEN,
        EFI_BACKGROUND_CYAN,
        EFI_BACKGROUND_RED,
        EFI_BACKGROUND_MAGENTA,
        EFI_BACKGROUND_BROWN,
        EFI_BACKGROUND_LIGHTGRAY,
    }, 
    IsSuccess, 
    UINTN,
    TRUE,
    FALSE,
};
use core::{cmp, mem::transmute};
use io::{self, Write, Cursor, BufRead, BufReader, LineWriter};
use ::Result;
use system_table;
use TextInputProcolPtr;
use alloc::{vec::Vec, string::String, str, fmt};

// TODO: This whole module has gotten ugly. Needs cleanup.
// TODO: Should we replace Console with two structs, StdIn and StdOut, corresponding to input and output? This is more in line with Rust stdlib.

#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum ForeColor {
    Black = EFI_BLACK,
    Blue = EFI_BLUE,
    Green = EFI_GREEN,
    Cyan = EFI_CYAN,
    Red = EFI_RED,
    Magenta = EFI_MAGENTA,
    Brown = EFI_BROWN,
    LightGray = EFI_LIGHTGRAY,
    DarkGray = EFI_DARKGRAY,
    LightBlue = EFI_LIGHTBLUE,
    LightGreen = EFI_LIGHTGREEN,
    LightCyan = EFI_LIGHTCYAN,
    LightRed = EFI_LIGHTRED,
    LightMagenta = EFI_LIGHTMAGENTA,
    Yellow = EFI_YELLOW,
    White = EFI_WHITE,
}

impl From<UINTN> for ForeColor {
    fn from(color_num: UINTN) -> Self {
        match color_num {
            EFI_BLACK..=EFI_WHITE => unsafe { transmute(color_num) },
            _ => panic!("Attempt to convert an out-of-range number to ForeColor")
        }
    }
}

impl From<ForeColor> for UINTN {
    fn from(fore_color: ForeColor) -> UINTN {
        fore_color as UINTN
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(usize)]
pub enum BackColor {
    Black = EFI_BACKGROUND_BLACK,
    Blue = EFI_BACKGROUND_BLUE,
    Green = EFI_BACKGROUND_GREEN,
    Cyan = EFI_BACKGROUND_CYAN,
    Red = EFI_BACKGROUND_RED,
    Magenta = EFI_BACKGROUND_MAGENTA,
    Brown = EFI_BACKGROUND_BROWN,
    LightGray = EFI_BACKGROUND_LIGHTGRAY,
}

impl From<UINTN> for BackColor {
    fn from(color_num: UINTN) -> Self {
        match color_num {
            EFI_BACKGROUND_BLACK..=EFI_BACKGROUND_LIGHTGRAY => unsafe { transmute(color_num) },
            _ => panic!("Attempt to convert an out-of-range number to BackColor")
        }
    }
}

impl From<BackColor> for UINTN {
    fn from(back_color: BackColor) -> UINTN {
        back_color as UINTN
    }
}

pub struct Console {
    pub input: TextInputProcolPtr,
    pub output: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    utf8_buf: io::Cursor<Vec<u8>>
}

const LF: u16 = 10;
const CR: u16 = 13;
const BS: u16 = 8;

impl Console {
    pub fn new(input: TextInputProcolPtr, output: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL) -> Self {
        Self { input, output, utf8_buf: Cursor::new(Vec::new()) }
    }

    pub fn cursor_pos(&self) -> Position {
        let mode = unsafe { (*(*self).output).Mode };
        Position { row: unsafe { (*mode).CursorRow } as u32, col: unsafe { (*mode).CursorColumn } as u32 } // To convert from i32 to u32 since screen coords can't be negative
    }

    pub fn set_cursor_pos(&self, pos: Position) -> Result<()> {
        unsafe {
            ret_on_err!(((*(*self).output).SetCursorPosition)(self.output, pos.col as usize, pos.row as usize));
        }

        Ok(())
    }

    pub fn enable_cursor(&mut self) -> Result<()> {
        unsafe {
            ret_on_err!(((*(*self).output).EnableCursor)(self.output, TRUE));
        }

        Ok(())
    }

    pub fn disable_cursor(&mut self) -> Result<()> {
        unsafe {
            ret_on_err!(((*(*self).output).EnableCursor)(self.output, FALSE));
        }

        Ok(())
    }

    pub fn clear_screen(&mut self) -> Result<()> {
        unsafe {
            ret_on_err!(((*(*self).output).ClearScreen)(self.output));
        }

        Ok(())
    }

    // TODO: instead of u32 use a strong type like 'ConsoleMode'
    // It may simply be a newtype around u32 and have methods
    // 'resolution()' which return something like 80x25, 80x50, Custom etc.
    pub fn max_supported_mode(&mut self) -> u32 {
        unsafe { (*(*(*self).output).Mode).MaxMode  as u32 } // Cast from i32 to u32 to is safe
    }

    pub fn set_mode(&mut self, mode_number: u32) -> Result<()> {
        unsafe {
            ret_on_err!(((*(*self).output).SetMode)(self.output, mode_number as usize)); // TODO: Cast should be safe on patforms with 32 and 64 ptr widths. Do we need to worry about other platforms?
        }

        Ok(())
    }

    pub fn fore_color(&mut self) -> ForeColor {
        let attribute = unsafe { (*(*(*self).output).Mode).Attribute } as UINTN; // TODO: Cast should be safe on patforms with 32 and 64 ptr widths. Do we need to worry about other platforms?
        let fore_color_num = attribute & 0b1111; // Bits 0..3 are fore color, 4..6 are back color

        fore_color_num.into()
    }

    pub fn set_fore_color(&mut self, fore_color: ForeColor) -> Result<()> {
        let curr_attribute = unsafe { (*(*(*self).output).Mode).Attribute } as UINTN; // TODO: Cast should be safe on patforms with 32 and 64 ptr widths. Do we need to worry about other platforms?
        let curr_back_color = curr_attribute & 0b1111_0000; // Bits 0..3 are fore color, 4..6 are back color
        let new_attribute = usize::from(fore_color) | curr_back_color;

        unsafe {
            ret_on_err!(((*(*self).output).SetAttribute)(self.output, new_attribute));
        }

        Ok(())
    }

    pub fn back_color(&mut self) -> BackColor {
        let attribute = unsafe { (*(*(*self).output).Mode).Attribute } as UINTN; // TODO: Cast should be safe on patforms with 32 and 64 ptr widths. Do we need to worry about other platforms?
        let back_color_num = attribute & 0b1111_0000; // Bits 0..3 are fore color, 4..6 are back color

        back_color_num.into()
    }

    pub fn set_back_color(&mut self, back_color: BackColor) -> Result<()> {
        let curr_attribute = unsafe { (*(*(*self).output).Mode).Attribute } as UINTN; // TODO: Cast should be safe on patforms with 32 and 64 ptr widths. Do we need to worry about other platforms?
        let curr_fore_color = curr_attribute & 0b1111; // Bits 0..3 are fore color, 4..6 are back color
        let new_attribute = curr_fore_color | usize::from(back_color);

        unsafe {
            ret_on_err!(((*(*self).output).SetAttribute)(self.output, new_attribute));
        }

        Ok(())
    }

    pub fn reset(&mut self, extended_verification: bool) -> Result<()> {
        unsafe {
            ret_on_err!(((*(*self).output).Reset)(self.output, if extended_verification { TRUE } else { FALSE }));
        }

        Ok(())
    }

    fn write_to_efi(&self, buf: &[u16]) -> Result<()> {
        unsafe {
            let (ptr, _) = to_ptr(buf);
            ret_on_err!(((*(*self).output).OutputString)(self.output, ptr));
            Ok(())
        }
    }

    fn read_from_efi(&self, buf: &mut [u16]) -> Result<usize> {
        match self.input {
            TextInputProcolPtr::Input(input) => self.read_from_efi_input(buf, input),
            TextInputProcolPtr::InputEx(input_ex) => self.read_from_efi_input_ex(buf, input_ex),
        }
    }

    // TODO: code in this function is super ugly and prone to bugs. Clean it up.
    fn read_from_efi_input_ex(&self, buf: &mut [u16], input_ex: *mut EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL) -> Result<usize> {
        let mut bytes_read = 0;

        let mut evt_index: UINTN = 0;
        let mut key_data = EFI_KEY_DATA::default();
        let mut evt_list = unsafe { [(*input_ex).WaitForKeyEx; 1] };

        while bytes_read < buf.len() {
            // TODO: For some reason we can't use ret_on_err here. Why?
            let status = unsafe { ((*system_table().BootServices).WaitForEvent)(evt_list.len(), evt_list.as_mut_ptr(), &mut evt_index) };
            if !IsSuccess(status) {
                return Err(status.into()); // TODO: Can we send some error text too with such errors
            }

            // TODO: For some reason we can't use ret_on_err here. Why?
            let status = unsafe { ((*input_ex).ReadKeyStrokeEx)(input_ex, &mut key_data) };
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

    fn read_from_efi_input(&self, buf: &mut [u16], input: *mut EFI_SIMPLE_TEXT_INPUT_PROTOCOL) -> Result<usize> {
        let mut bytes_read = 0;

        let mut evt_index: UINTN = 0;
        let mut key_data = EFI_INPUT_KEY::default();
        let mut evt_list = unsafe { [(*input).WaitForKey; 1] };

        while bytes_read < buf.len() {
            // TODO: For some reason we can't use ret_on_err here. Why?
            let status = unsafe { ((*system_table().BootServices).WaitForEvent)(evt_list.len(), evt_list.as_mut_ptr(), &mut evt_index) };
            if !IsSuccess(status) {
                return Err(status.into()); // TODO: Can we send some error text too with such errors
            }

            // TODO: For some reason we can't use ret_on_err here. Why?
            let status = unsafe { ((*input).ReadKeyStroke)(input, &mut key_data) };
            if !IsSuccess(status) {
                return Err(status.into()); // TODO: Can we send some error text too with such errors
            }

            if key_data.UnicodeChar != 0 { // != 0 means it's a printable unicode char
                if key_data.UnicodeChar == CR { // Safe to check for CR only without waiting for LF because in my experience UEFI only ever inserts CR when you press the Enter key
                    key_data.UnicodeChar = LF; // Always translate CR's to LF's to normalize line endings.
                }

                match key_data.UnicodeChar {
                    BS => {
                        if bytes_read > 0 {
                            bytes_read -= 1;
                            self.write_to_efi(&[BS, 0])?; // 0 is for null termination
                        }
                    },
                    c => {
                        buf[bytes_read] = c;
                        bytes_read += 1;

                        if c == LF {
                            self.write_to_efi(&[CR, LF, 0])?; // Must echo both CR and LF because other wise it fucks up the cursor position.
                            break;
                        } else {
                            self.write_to_efi(&[c, 0])?;
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

// TODO: write! works but writeln! doesn't. It doesn't result in an carriage returns at the end even though we're normalizing line endings in write() below. Fix this.
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
        let mut expected_utf16_buf_size = utf16_iter.size_hint().1.unwrap_or(utf8_buf.len()); // Guessing the capacity of utf16 buffer.
        let five_percent = (expected_utf16_buf_size as f32 * 0.05) as usize;
        let extra_size_for_line_endings = cmp::max(5, five_percent); // Least of 5 chars worth of space will come into play for very small writes. Without min limit extra could come out to be zero.
        expected_utf16_buf_size += extra_size_for_line_endings; // Extra added in case we have to normalize line endings
        let mut utf16_buf = Vec::with_capacity(expected_utf16_buf_size);

        let mut last_c = 0_u16;
        for (i, c) in utf16_iter.enumerate() {
            if c == LF && (i == 0 || last_c != CR) { // Normalizing LF's
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

// TODO: Implement StdErr
pub struct StdIn(BufReader<Console>);

impl StdIn {
    fn new(c: Console) -> Self {
        StdIn(BufReader::new(c))
    }
}

impl io::Read for StdIn {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl BufRead for StdIn {
    fn fill_buf(&mut self) -> io::Result<&[u8]> { self.0.fill_buf() }
    fn consume(&mut self, n: usize) { self.0.consume(n) }
}

pub struct StdOut(LineWriter<Console>);

impl StdOut {
    fn new(c: Console) -> Self {
        StdOut(LineWriter::new(c))
    }
}

impl io::Write for StdOut {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub row: u32,
    pub col: u32
}

pub fn console() -> Console {
    ::SystemTable::new(system_table())
        .expect("failed to create system table").console()
}

// TODO: Remove this uncessary SystemTable::new() business
// Do we need this SystemTable type?
pub fn stdin() -> StdIn {
    StdIn::new(console())
}

pub fn stdout() -> StdOut {
    StdOut::new(console())
}

#[macro_export]
macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::print_args(format_args!($($arg)*)));
}

// TODO: Call to stdout() creates a new StdOut obj everytime. Remove this extravagance.
pub fn print_args(args: fmt::Arguments) {
    return stdout().write_fmt(args).expect("Failed to write to stdout")
}


fn invalid_encoding() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "text was not valid unicode")
}

fn to_ptr<T>(slice: &[T]) -> (*const T, usize) {
    unsafe {
        transmute(slice)
    }
}
