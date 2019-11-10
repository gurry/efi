
use core::{mem, fmt};

/// EFI Time Abstraction:
///  Year:       1900 - 9999
///  Month:      1 - 12
///  Day:        1 - 31
///  Hour:       0 - 23
///  Minute:     0 - 59
///  Second:     0 - 59
///  Nanosecond: 0 - 999,999,999
///  TimeZone:   -1440 to 1440 or 2047
#[repr(C)]
#[derive(Debug)]
pub struct EFI_TIME {
    pub Year:   UINT16,
    pub Month: UINT8,
    pub Day: UINT8,
    pub Hour: UINT8,
    pub Minute: UINT8,
    pub Second: UINT8,
    pub Pad1: UINT8,
    pub Nanosecond: UINT32,
    pub TimeZone: INT16,
    pub Daylight: UINT8,
    pub Pad2: UINT8,
}

impl EFI_TIME {
    pub fn zero() -> Self {
        EFI_TIME {
            Year: 0,
            Month: 0,
            Day: 0,
            Hour: 0,
            Minute: 0,
            Second: 0,
            Pad1: 0,
            Nanosecond: 0,
            TimeZone: 0,
            Daylight: 0,
            Pad2: 0,
        }
    }
}

impl Default for EFI_TIME {
    fn default() -> Self {
        Self::zero()
    }
}

pub const EFI_TIME_ADJUST_DAYLIGHT: UINTN = 0x01;
pub const EFI_TIME_IN_DAYLIGHT: UINTN = 0x02;
pub const EFI_UNSPECIFIED_TIMEZONE: UINTN = 0x07FF;

#[repr(C)]
#[derive(Debug)]
pub struct EFI_TIME_CAPABILITIES {
    pub Resolution: UINT32,
    pub Accuracy: UINT32,
    pub SetsToZero: BOOLEAN,
}

impl EFI_TIME_CAPABILITIES  {
    pub fn zero() -> Self {
        Self {
            Resolution: 0,
            Accuracy: 0,
            SetsToZero: 0,
        }
    }
}

/// 4-byte buffer. An IPv4 internet protocol address.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct EFI_IPv4_ADDRESS {
  pub Addr: [UINT8; 4],
}

impl EFI_IPv4_ADDRESS {
    pub fn zero() -> Self {
        Self { Addr: [0, 0, 0, 0] }
    }
}

/// 16-byte buffer. An IPv6 internet protocol address.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct EFI_IPv6_ADDRESS {
  pub Addr: [UINT8; 16],
}

impl EFI_IPv6_ADDRESS {
    pub fn zero() -> Self {
        Self { Addr: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }
    }
}

/// 32-byte buffer containing a network Media Access Control address.
#[derive(PartialEq, Eq)]
#[repr(C)]
pub struct EFI_MAC_ADDRESS {
  pub Addr: [UINT8; 32],
}

impl EFI_MAC_ADDRESS  {
    pub fn zero() -> Self {
        Self { Addr: [0; 32] }
    }
}

impl Default for EFI_MAC_ADDRESS {
    fn default() -> Self {
        Self::zero()
    }
}

// Had to implement by hand 'cause Debug derive not allowed for unions
impl fmt::Debug for EFI_MAC_ADDRESS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EFI_MAC_ADDRESS ")
            .field("Addr", &format_args!("{:x?}", self.Addr))
            .finish()
    }
}


/// 16-byte buffer aligned on a 4-byte boundary.
/// An IPv4 or IPv6 internet protocol address.
#[derive(Copy, Clone)]
#[repr(C)]
pub union EFI_IP_ADDRESS {
  pub Addr: [UINT32; 4],
  pub v4: EFI_IPv4_ADDRESS,
  pub v6: EFI_IPv6_ADDRESS,
}

impl EFI_IP_ADDRESS {
    pub fn zero() -> Self {
        Self { Addr: [0; 4] }
    }
}

// Had to implement by hand 'cause Debug derive not allowed for unions
impl fmt::Debug for EFI_IP_ADDRESS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EFI_IP_ADDRESS")
            .field("Addr", &format_args!("{:?}", unsafe { self.Addr }))
            .finish()
    }
}

pub type EFI_PHYSICAL_ADDRESS = UINT64;

#[derive(Debug)]
#[repr(C)]
pub struct EFI_TABLE_HEADER {
    Signature : UINT64,
    Revision : UINT32,
    HeaderSize : UINT32,
    CRC32 : UINT32,
    Reserved : UINT32
}

macro_rules! with_high_bit_set {
    ($num:expr) => { (1 << ((mem::size_of::<UINTN>() * 8) - 1)) | $num };
}

pub const EFI_SUCCESS: UINTN = 0; // The operation completed successfully.
pub const EFI_LOAD_ERROR: UINTN = with_high_bit_set!(1); // The image failed to load.
pub const EFI_INVALID_PARAMETER: UINTN = with_high_bit_set!(2); // A parameter was incorrect.
pub const EFI_UNSUPPORTED: UINTN = with_high_bit_set!(3); // The operation is not supported.
pub const EFI_BAD_BUFFER_SIZE: UINTN = with_high_bit_set!(4); // The buffer was not the proper size for the request.
pub const EFI_BUFFER_TOO_SMALL: UINTN = with_high_bit_set!(5); // The buffer is not large enough to hold the requested data.The required buffer size is returned in the appropriate parameter when this error occurs.
pub const EFI_NOT_READY: UINTN = with_high_bit_set!(6); // There is no data pending upon return.
pub const EFI_DEVICE_ERROR: UINTN = with_high_bit_set!(7); // The physical device reported an error while attempting the operation.
pub const EFI_WRITE_PROTECTED: UINTN = with_high_bit_set!(8); // The device cannot be written to.
pub const EFI_OUT_OF_RESOURCES: UINTN = with_high_bit_set!(9); // A resource has run out.
pub const EFI_VOLUME_CORRUPTED: UINTN = with_high_bit_set!(10); // An inconstancy was detected on the file system causing the operating to fail.
pub const EFI_VOLUME_FULL: UINTN = with_high_bit_set!(11); // There is no more space on the file system.
pub const EFI_NO_MEDIA: UINTN = with_high_bit_set!(12); // The device does not contain any medium to perform the operation.
pub const EFI_MEDIA_CHANGED: UINTN = with_high_bit_set!(13); // The medium in the device has changed since the last access.
pub const EFI_NOT_FOUND: UINTN = with_high_bit_set!(14); // The item was not found.
pub const EFI_ACCESS_DENIED: UINTN = with_high_bit_set!(15); // Access was denied.
pub const EFI_NO_RESPONSE: UINTN = with_high_bit_set!(16); // The server was not found or did not respond to the request.
pub const EFI_NO_MAPPING: UINTN = with_high_bit_set!(17); // A mapping to a device does not exist.
pub const EFI_TIMEOUT: UINTN = with_high_bit_set!(18); // The timeout time expired.
pub const EFI_NOT_STARTED: UINTN = with_high_bit_set!(19); // The protocol has not been started.
pub const EFI_ALREADY_STARTED: UINTN = with_high_bit_set!(20); // The protocol has already been started.
pub const EFI_ABORTED: UINTN = with_high_bit_set!(21); // The operation was aborted.
pub const EFI_ICMP_ERROR: UINTN = with_high_bit_set!(22); // An ICMP error occurred during the network operation.
pub const EFI_TFTP_ERROR: UINTN = with_high_bit_set!(23); // A TFTP error occurred during the network operation.
pub const EFI_PROTOCOL_ERROR: UINTN = with_high_bit_set!(24); // A protocol error occurred during the network operation.
pub const EFI_INCOMPATIBLE_VERSION: UINTN = with_high_bit_set!(25); // The function encountered an internal version that was incompatible with a version requested by the caller.
pub const EFI_SECURITY_VIOLATION: UINTN = with_high_bit_set!(26); // The function was not performed due to a security violation.
pub const EFI_CRC_ERROR: UINTN = with_high_bit_set!(27); // A CRC error was detected.
pub const EFI_END_OF_MEDIA: UINTN = with_high_bit_set!(28); // Beginning or end of media was reached
pub const EFI_END_OF_FILE: UINTN = with_high_bit_set!(31); // The end of the file was reached.
pub const EFI_INVALID_LANGUAGE: UINTN = with_high_bit_set!(32); // The language specified was invalid.
pub const EFI_COMPROMISED_DATA: UINTN = with_high_bit_set!(33); // The security status of the data is unknown or compromisedand the data must be updated or replaced to restore a valid security status.
pub const EFI_IP_ADDRESS_CONFLICT: UINTN = with_high_bit_set!(34); // There is an address conflict address allocation


pub const EFI_WARN_UNKNOWN_GLYPH: UINTN = 1; // The string contained one or more characters that the device could not render and were skipped.
pub const EFI_WARN_DELETE_FAILURE: UINTN = 2; // The handle was closed, but the file was not deleted.
pub const EFI_WARN_WRITE_FAILURE: UINTN = 3; // The handle was closed, but the data to the file was not flushed properly.
pub const EFI_WARN_BUFFER_TOO_SMALL: UINTN = 4; // The resulting buffer was too small, and the data was truncated to the buffer size.
pub const EFI_WARN_STALE_DATA: UINTN = 5; // The data has not been updated within the timeframe set by local policy for this type of data.

#[derive(Debug, PartialEq, Eq)]
pub enum EFI_STATUS_TYPE {
    SUCCESS,
    ERROR,
    WARNING
}

fn has_high_bit_set(value: UINTN) -> bool {
    ((1 << ((mem::size_of::<UINTN>() * 8) - 1)) & value) != 0
}

pub fn StatusType(status: EFI_STATUS) -> EFI_STATUS_TYPE {
    match status {
        EFI_SUCCESS => EFI_STATUS_TYPE::SUCCESS,
        s if has_high_bit_set(s) => EFI_STATUS_TYPE::ERROR,
        _ => EFI_STATUS_TYPE::WARNING
    }
}

pub fn IsSuccess(status: EFI_STATUS) -> bool {
    StatusType(status) == EFI_STATUS_TYPE::SUCCESS
}

pub fn IsError(status: EFI_STATUS) -> bool {
    StatusType(status) == EFI_STATUS_TYPE::ERROR
}
pub fn IsWarning(status: EFI_STATUS) -> bool {
    StatusType(status) == EFI_STATUS_TYPE::WARNING
}



pub type UINT64 = u64;
pub type INT64 = i64;
pub type UINT32 = u32;
pub type INT32 = i32;
pub type UINT16 = u16;
pub type CHAR16 = u16;
pub type INT16 = i16;
pub type BOOLEAN = u8;
pub type UINT8 = u8;
pub type CHAR8 = i8;
pub type INT8 = i8;
pub type UINTN = usize;

pub const TRUE: BOOLEAN = 1;
pub const FALSE: BOOLEAN = 0;

#[derive(Copy, Clone)]
pub enum VOID { }
pub type EFI_HANDLE = *const VOID;
pub type EFI_EVENT = *const VOID;

#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
pub struct EFI_GUID(pub UINT32, pub UINT16, pub UINT16, pub [UINT8; 8]);

pub type EFI_STATUS = UINTN;

pub enum NOT_DEFINED  {}
