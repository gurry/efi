use ffi::base::*;

// OUTPUT PROTOCOL
pub const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x387477c2, 0x69c7, 0x11d2, [0x8e, 0x39, 0x0, 0xa0, 0xc9, 0x69, 0x72, 0x3b]);

pub const BOXDRAW_HORIZONTAL: UINTN = 0x2500;
pub const BOXDRAW_VERTICAL: UINTN = 0x2502;
pub const BOXDRAW_DOWN_RIGHT: UINTN = 0x250c;
pub const BOXDRAW_DOWN_LEFT: UINTN = 0x2510;
pub const BOXDRAW_UP_RIGHT: UINTN = 0x2514;
pub const BOXDRAW_UP_LEFT: UINTN = 0x2518;
pub const BOXDRAW_VERTICAL_RIGHT: UINTN = 0x251c;
pub const BOXDRAW_VERTICAL_LEFT: UINTN = 0x2524;
pub const BOXDRAW_DOWN_HORIZONTAL: UINTN = 0x252c;
pub const BOXDRAW_UP_HORIZONTAL: UINTN = 0x2534;
pub const BOXDRAW_VERTICAL_HORIZONTAL: UINTN = 0x253c;
pub const BOXDRAW_DOUBLE_HORIZONTAL: UINTN = 0x2550;
pub const BOXDRAW_DOUBLE_VERTICAL: UINTN = 0x2551;
pub const BOXDRAW_DOWN_RIGHT_DOUBLE: UINTN = 0x2552;
pub const BOXDRAW_DOWN_DOUBLE_RIGHT: UINTN = 0x2553;
pub const BOXDRAW_DOUBLE_DOWN_RIGHT: UINTN = 0x2554;
pub const BOXDRAW_DOWN_LEFT_DOUBLE: UINTN = 0x2555;
pub const BOXDRAW_DOWN_DOUBLE_LEFT: UINTN = 0x2556;
pub const BOXDRAW_DOUBLE_DOWN_LEFT: UINTN = 0x2557;
pub const BOXDRAW_UP_RIGHT_DOUBLE: UINTN = 0x2558;
pub const BOXDRAW_UP_DOUBLE_RIGHT: UINTN = 0x2559;
pub const BOXDRAW_DOUBLE_UP_RIGHT: UINTN = 0x255a;
pub const BOXDRAW_UP_LEFT_DOUBLE: UINTN = 0x255b;
pub const BOXDRAW_UP_DOUBLE_LEFT: UINTN = 0x255c;
pub const BOXDRAW_DOUBLE_UP_LEFT: UINTN = 0x255d;
pub const BOXDRAW_VERTICAL_RIGHT_DOUBLE: UINTN = 0x255e;
pub const BOXDRAW_VERTICAL_DOUBLE_RIGHT: UINTN = 0x255f;
pub const BOXDRAW_DOUBLE_VERTICAL_RIGHT: UINTN = 0x2560;
pub const BOXDRAW_VERTICAL_LEFT_DOUBLE: UINTN = 0x2561;
pub const BOXDRAW_VERTICAL_DOUBLE_LEFT: UINTN = 0x2562;
pub const BOXDRAW_DOUBLE_VERTICAL_LEFT: UINTN = 0x2563;
pub const BOXDRAW_DOWN_HORIZONTAL_DOUBLE: UINTN = 0x2564;
pub const BOXDRAW_DOWN_DOUBLE_HORIZONTAL: UINTN = 0x2565;
pub const BOXDRAW_DOUBLE_DOWN_HORIZONTAL: UINTN = 0x2566;
pub const BOXDRAW_UP_HORIZONTAL_DOUBLE: UINTN = 0x2567;
pub const BOXDRAW_UP_DOUBLE_HORIZONTAL: UINTN = 0x2568;
pub const BOXDRAW_DOUBLE_UP_HORIZONTAL: UINTN = 0x2569;
pub const BOXDRAW_VERTICAL_HORIZONTAL_DOUBLE: UINTN = 0x256a;
pub const BOXDRAW_VERTICAL_DOUBLE_HORIZONTAL: UINTN = 0x256b;
pub const BOXDRAW_DOUBLE_VERTICAL_HORIZONTAL: UINTN = 0x256c;


pub const BLOCKELEMENT_FULL_BLOCK: UINTN = 0x2588;
pub const BLOCKELEMENT_LIGHT_SHADE: UINTN = 0x2591;


pub const GEOMETRICSHAPE_UP_TRIANGLE: UINTN = 0x25b2;
pub const GEOMETRICSHAPE_RIGHT_TRIANGLE: UINTN = 0x25ba;
pub const GEOMETRICSHAPE_DOWN_TRIANGLE: UINTN = 0x25bc;
pub const GEOMETRICSHAPE_LEFT_TRIANGLE: UINTN = 0x25c4;


pub const ARROW_LEFT: UINTN = 0x2190;
pub const ARROW_UP: UINTN = 0x2191;
pub const ARROW_RIGHT: UINTN = 0x2192;
pub const ARROW_DOWN: UINTN = 0x2193;


pub const EFI_BLACK: UINTN = 0x00;
pub const EFI_BLUE: UINTN = 0x01;
pub const EFI_GREEN: UINTN = 0x02;
pub const EFI_CYAN: UINTN = (EFI_BLUE | EFI_GREEN);
pub const EFI_RED: UINTN = 0x04;
pub const EFI_MAGENTA: UINTN = (EFI_BLUE | EFI_RED);
pub const EFI_BROWN: UINTN = (EFI_GREEN | EFI_RED);
pub const EFI_LIGHTGRAY: UINTN = (EFI_BLUE | EFI_GREEN | EFI_RED);
pub const EFI_BRIGHT: UINTN = 0x08;
pub const EFI_DARKGRAY: UINTN = (EFI_BRIGHT);
pub const EFI_LIGHTBLUE: UINTN = (EFI_BLUE | EFI_BRIGHT);
pub const EFI_LIGHTGREEN: UINTN = (EFI_GREEN | EFI_BRIGHT);
pub const EFI_LIGHTCYAN: UINTN = (EFI_CYAN | EFI_BRIGHT);
pub const EFI_LIGHTRED: UINTN = (EFI_RED | EFI_BRIGHT);
pub const EFI_LIGHTMAGENTA: UINTN = (EFI_MAGENTA | EFI_BRIGHT);
pub const EFI_YELLOW: UINTN = (EFI_BROWN | EFI_BRIGHT);
pub const EFI_WHITE: UINTN = (EFI_BLUE | EFI_GREEN | EFI_RED | EFI_BRIGHT);

// #define EFI_TEXT_ATTR(f, b)       ((f) | ((b) << 4))

pub const EFI_BACKGROUND_BLACK: UINTN = 0x00;
pub const EFI_BACKGROUND_BLUE: UINTN = 0x10;
pub const EFI_BACKGROUND_GREEN: UINTN = 0x20;
pub const EFI_BACKGROUND_CYAN: UINTN = (EFI_BACKGROUND_BLUE | EFI_BACKGROUND_GREEN);
pub const EFI_BACKGROUND_RED: UINTN = 0x40;
pub const EFI_BACKGROUND_MAGENTA: UINTN = (EFI_BACKGROUND_BLUE | EFI_BACKGROUND_RED);
pub const EFI_BACKGROUND_BROWN: UINTN = (EFI_BACKGROUND_GREEN | EFI_BACKGROUND_RED);
pub const EFI_BACKGROUND_LIGHTGRAY: UINTN = (EFI_BACKGROUND_BLUE | EFI_BACKGROUND_GREEN | EFI_BACKGROUND_RED);

// We currently define attributes from 0 - 7F for color manipulations
// To internally handle the local display characteristics for a particular character, 
// Bit 7 signifies the local glyph representation for a character.  If turned on, glyphs will be
// pulled from the wide glyph database and will display locally as a wide character (16 X 19 versus 8 X 19)
// If bit 7 is off, the narrow glyph database will be used.  This does NOT affect information that is sent to
// non-local displays, such as serial or LAN consoles.
pub const EFI_WIDE_ATTRIBUTE: UINTN = 0x80;

pub type EFI_TEXT_RESET = extern "win64" fn(
    This: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    ExtendedVerification: BOOLEAN
) -> EFI_STATUS;

pub type EFI_TEXT_STRING = extern "win64" fn(
    This: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    String: *const CHAR16
 ) -> EFI_STATUS;

pub type EFI_TEXT_TEST_STRING = extern "win64" fn(
    This: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    String: *const CHAR16
) -> EFI_STATUS;

pub type EFI_TEXT_QUERY_MODE = extern "win64" fn(
    This: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    ModeNumber: UINTN,
    Columns: *const UINTN,
    Rows: *const UINTN
) -> EFI_STATUS;

pub type EFI_TEXT_SET_MODE = extern "win64" fn(
    This: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    ModeNumber: UINTN
) -> EFI_STATUS;

pub type EFI_TEXT_SET_ATTRIBUTE = extern "win64" fn(
    This: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    Attribute: UINTN
) -> EFI_STATUS;

pub type EFI_TEXT_CLEAR_SCREEN = extern "win64" fn(
    This: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL
) -> EFI_STATUS;

pub type EFI_TEXT_SET_CURSOR_POSITION = extern "win64" fn(
    This: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    Column: UINTN,
    Row: UINTN
) -> EFI_STATUS;

pub type EFI_TEXT_ENABLE_CURSOR = extern "win64" fn(
    This: *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    Visible: BOOLEAN
) -> EFI_STATUS;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct EFI_SIMPLE_TEXT_OUTPUT_MODE {
  pub MaxMode: INT32,
  pub Mode: INT32,
  pub Attribute: INT32,
  pub CursorColumn: INT32,
  pub CursorRow: INT32,
  pub CursorVisible: BOOLEAN,
}

impl Default for EFI_SIMPLE_TEXT_OUTPUT_MODE  {
    fn default() -> Self { 
        Self {
            MaxMode: 0,
            Mode: 0,
            Attribute: 0,
            CursorColumn: 0,
            CursorRow: 0,
            CursorVisible: FALSE,
        }
    }
}

#[repr(C)]
pub struct EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL {
  pub Reset: EFI_TEXT_RESET,
  pub OutputString: EFI_TEXT_STRING,
  pub TestString: EFI_TEXT_TEST_STRING,
  pub QueryMode: EFI_TEXT_QUERY_MODE,
  pub SetMode: EFI_TEXT_SET_MODE,
  pub SetAttribute: EFI_TEXT_SET_ATTRIBUTE,
  pub ClearScreen: EFI_TEXT_CLEAR_SCREEN,
  pub SetCursorPosition: EFI_TEXT_SET_CURSOR_POSITION,
  pub EnableCursor: EFI_TEXT_ENABLE_CURSOR,
  pub Mode: *const EFI_SIMPLE_TEXT_OUTPUT_MODE,
}


// INPUT PROTOCOL
pub const EFI_SIMPLE_TEXT_INPUT_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0x387477c1, 0x69c7, 0x11d2, [0x8e, 0x39, 0x00, 0xa0, 0xc9, 0x69, 0x72, 0x3b]);

pub type EFI_INPUT_RESET = extern "win64" fn(
    This: *mut EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
    ExtendedVerification: BOOLEAN
) -> EFI_STATUS;

pub type EFI_INPUT_READ_KEY = extern "win64" fn(
    This: *mut EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
    Key: *mut EFI_INPUT_KEY
) -> EFI_STATUS;

#[repr(C)]
#[derive(Debug)]
pub struct EFI_INPUT_KEY {
    pub ScanCode: UINT16,
    pub UnicodeChar: CHAR16
}

impl Default for EFI_INPUT_KEY  {
    fn default() -> Self {
        Self {
            ScanCode: 0,
            UnicodeChar: 0
        }
    }
}

#[repr(C)]
pub struct EFI_SIMPLE_TEXT_INPUT_PROTOCOL {
    pub Reset: EFI_INPUT_RESET,
    pub ReadKeyStroke: EFI_INPUT_READ_KEY,
    pub WaitForKey: EFI_EVENT,
}

pub const EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL_GUID: EFI_GUID = EFI_GUID(0xdd9e7534, 0x7762, 0x4698, [0x8c, 0x14, 0xf5, 0x85, 0x17, 0xa6, 0x25, 0xaa]);

#[repr(C)]
pub struct EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL{
    pub Reset: EFI_INPUT_RESET_EX,
    pub ReadKeyStrokeEx: EFI_INPUT_READ_KEY_EX,
    pub WaitForKeyEx: EFI_EVENT,
    pub SetState: EFI_SET_STATE,
    pub RegisterKeyNotify: EFI_REGISTER_KEYSTROKE_NOTIFY,
    pub UnregisterKeyNotify: EFI_UNREGISTER_KEYSTROKE_NOTIFY,
}

pub type EFI_INPUT_RESET_EX = extern "win64" fn(
    This: *mut EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL,
    ExtendedVerification: BOOLEAN
) -> EFI_STATUS;

pub type EFI_INPUT_READ_KEY_EX = extern "win64" fn(
    This: *mut EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL,
    KeyData: *mut EFI_KEY_DATA
) -> EFI_STATUS;

pub type EFI_SET_STATE = extern "win64" fn(
    This: *mut EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL,
    KeyToggleState: *const EFI_KEY_TOGGLE_STATE 
) -> EFI_STATUS;

pub type EFI_REGISTER_KEYSTROKE_NOTIFY = extern "win64" fn(
    This: *mut EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL,
    KeyData: *const EFI_KEY_DATA,
    KeyNotificationFunction: EFI_KEY_NOTIFY_FUNCTION,
    NotifyHandle: *const *mut VOID
) -> EFI_STATUS;

pub type EFI_KEY_NOTIFY_FUNCTION = extern "win64" fn(
    KeyData: *const EFI_KEY_DATA
) -> EFI_STATUS;

pub type EFI_UNREGISTER_KEYSTROKE_NOTIFY = extern "win64" fn(
    This: *mut EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL,
    NotificationHandle: *const VOID
);

#[repr(C)]
pub struct EFI_KEY_DATA {
    pub Key: EFI_INPUT_KEY,
    pub KeyState: EFI_KEY_STATE
}

impl Default for EFI_KEY_DATA {
    fn default() -> Self {
        Self {
            Key: EFI_INPUT_KEY::default(),
            KeyState: EFI_KEY_STATE::default()
        }
    }
}

pub const EFI_SHIFT_STATE_VALID: UINT32 = 0x80000000;
pub const EFI_RIGHT_SHIFT_PRESSED: UINT32 = 0x00000001;
pub const EFI_LEFT_SHIFT_PRESSED: UINT32 = 0x00000002;
pub const EFI_RIGHT_CONTROL_PRESSED: UINT32 = 0x00000004;
pub const EFI_LEFT_CONTROL_PRESSED: UINT32 = 0x00000008;
pub const EFI_RIGHT_ALT_PRESSED: UINT32 = 0x00000010;
pub const EFI_LEFT_ALT_PRESSED: UINT32 = 0x00000020;
pub const EFI_RIGHT_LOGO_PRESSED: UINT32 = 0x00000040;
pub const EFI_LEFT_LOGO_PRESSED: UINT32 = 0x00000080;
pub const EFI_MENU_KEY_PRESSED: UINT32 = 0x00000100;
pub const EFI_SYS_REQ_PRESSED: UINT32 = 0x00000200;

pub type EFI_KEY_TOGGLE_STATE = UINT8;

pub const EFI_TOGGLE_STATE_VALID: EFI_KEY_TOGGLE_STATE = 0x80;
pub const EFI_KEY_STATE_EXPOSED: EFI_KEY_TOGGLE_STATE  = 0x40;
pub const EFI_SCROLL_LOCK_ACTIVE: EFI_KEY_TOGGLE_STATE = 0x01;
pub const EFI_NUM_LOCK_ACTIVE: EFI_KEY_TOGGLE_STATE = 0x02;
pub const EFI_CAPS_LOCK_ACTIVE: EFI_KEY_TOGGLE_STATE = 0x04;

pub struct EFI_KEY_STATE {
    pub KeyShiftState: UINT32,
    pub KeyToggleState: EFI_KEY_TOGGLE_STATE,
}

impl Default for EFI_KEY_STATE  {
    fn default() -> Self {
        Self {
            KeyShiftState: 0x00,
            KeyToggleState: 0x00
        }
    }
}