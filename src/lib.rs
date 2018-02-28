// TODO: Can we use core::intrinsics (https://doc.rust-lang.org/1.12.0/core/intrinsics/fn.copy.html) instead of compiler_builtins crate?
#![no_std]
#![feature(intrinsics)]

#[macro_use] extern crate failure;

pub mod ffi;
pub mod boot_services;
pub mod protocols;

// Hack: this std declartion is to work around a bug in failure crate
// wherein it looks for std even in no_std crates. Will remove it when
// the bug is fixed.
mod std {
    pub use core::option;
    pub use core::fmt;
}

use core::fmt::{Debug, Display, Formatter};
use ffi::{EFI_STATUS, EFI_SYSTEM_TABLE};
use core::mem::transmute;
use protocols::console::Console;
use failure::{Context, Fail, Backtrace};


pub struct EfiError {
    inner: Context<EfiErrorKind>
}

impl EfiError {
    pub fn kind(&self) -> EfiErrorKind {
        *self.inner.get_context()
    }
}

impl From<EfiErrorKind> for EfiError {
    fn from(kind: EfiErrorKind) -> EfiError {
        EfiError { inner: Context::new(kind) }
    }
}

impl From<Context<EfiErrorKind>> for EfiError {
    fn from(inner: Context<EfiErrorKind>) -> EfiError {
        EfiError { inner: inner }
    }
}

impl From<EFI_STATUS> for EfiError {
    fn from(status: ffi::EFI_STATUS) -> Self {
        EfiError::from(EfiErrorKind::from(status))
    }
}

impl Fail for EfiError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Debug for EfiError {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{:?} (0x{:X})", self.kind() , self.kind() as usize)
    }
}

impl Display for EfiError {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{:?} (0x{:X}) - {}", self.kind() , self.kind() as usize, self.kind())
    }
}

#[derive(Debug, Fail, Copy, Clone)]
#[repr(usize)]
pub enum EfiErrorKind {
    #[fail(display = "The image failed to load")]
    LoadError = ffi::EFI_LOAD_ERROR,
    #[fail(display = "A parameter was incorrect")]
    InvalidParameter = ffi::EFI_INVALID_PARAMETER,
    #[fail(display = "The operation is not supported")]
    Unsupported = ffi::EFI_UNSUPPORTED,
    #[fail(display = "The buffer was not the proper size for the request")]
    BadBufferSize = ffi::EFI_BAD_BUFFER_SIZE,
    #[fail(display = "The buffer is not large enough to hold the requested data")]
    BufferTooSmall = ffi::EFI_BUFFER_TOO_SMALL,
    #[fail(display = "There is no data pending upon return")]
    NotReady = ffi::EFI_NOT_READY,
    #[fail(display = "The physical device reported an error while attempting the operation")]
    DeviceError = ffi::EFI_DEVICE_ERROR,
    #[fail(display = "The device cannot be written to")]
    WriteProtected = ffi::EFI_WRITE_PROTECTED,
    #[fail(display = "A resource has run out")]
    OutOfResources = ffi::EFI_OUT_OF_RESOURCES,
    #[fail(display = "An inconstency was detected on the file system causing the operation to fail")]
    VolumeCorrupted = ffi::EFI_VOLUME_CORRUPTED,
    #[fail(display = "There is no more space on the file system")]
    VolumeFull = ffi::EFI_VOLUME_FULL,
    #[fail(display = "The device does not contain any medium to perform the operation")]
    NoMedia = ffi::EFI_NO_MEDIA,
    #[fail(display = "The medium in the device has changed since the last access")]
    MediaChanged = ffi::EFI_MEDIA_CHANGED,
    #[fail(display = "The item was not found")]
    NotFound = ffi::EFI_NOT_FOUND,
    #[fail(display = "Access was denied")]
    AccessDenied = ffi::EFI_ACCESS_DENIED,
    #[fail(display = "The server was not found or did not respond to the request")]
    NoResponse = ffi::EFI_NO_RESPONSE,
    #[fail(display = "A mapping to a device does not exist")]
    NoMapping = ffi::EFI_NO_MAPPING,
    #[fail(display = "The timeout time expired")]
    Timeout = ffi::EFI_TIMEOUT,
    #[fail(display = "The protocol has not been started")]
    NotStarted = ffi::EFI_NOT_STARTED,
    #[fail(display = "The protocol has already been started")]
    AlreadyStarted = ffi::EFI_ALREADY_STARTED,
    #[fail(display = "The operation was aborted")]
    Aborted = ffi::EFI_ABORTED,
    #[fail(display = "An ICMP error occurred during the network operation")]
    IcmpError = ffi::EFI_ICMP_ERROR,
    #[fail(display = "A TFTP error occurred during the network operation")]
    TftpError = ffi::EFI_TFTP_ERROR,
    #[fail(display = "A protocol error occurred during the network operation")]
    ProtocolError = ffi::EFI_PROTOCOL_ERROR,
    #[fail(display = "The function encountered an internal version that was incompatible with a version requested by the caller")]
    IncompatibleVersion = ffi::EFI_INCOMPATIBLE_VERSION,
    #[fail(display = "The function was not performed due to a security violation")]
    SecurityViolation = ffi::EFI_SECURITY_VIOLATION,
    #[fail(display = "A CRC error was detected")]
    CrcError = ffi::EFI_CRC_ERROR,
    #[fail(display = "Beginning or end of media was reached")]
    EndOfMedia = ffi::EFI_END_OF_MEDIA,
    #[fail(display = "The end of the file was reached")]
    EndOfFile = ffi::EFI_END_OF_FILE,
    #[fail(display = "The language specified was invalid")]
    InvalidLanguage = ffi::EFI_INVALID_LANGUAGE,
    #[fail(display = "The security status of the data is unknown or compromised and the data must be updated or replaced to restore a valid security status")]
    CompromisedData = ffi::EFI_COMPROMISED_DATA,
    #[fail(display = "There is an address conflict during address allocation")]
    IpAddressConflict = ffi::EFI_IP_ADDRESS_CONFLICT,
    #[fail(display = "Unrecognized EFI error")]
    UnrecognizedError = <EFI_STATUS>::max_value()
}

impl From<EFI_STATUS> for EfiErrorKind {
    fn from(status: ffi::EFI_STATUS) -> Self {
        if ffi::IsError(status) { unsafe { transmute(status) } } else { EfiErrorKind::UnrecognizedError }
    }
}

impl Into<usize> for EfiErrorKind {
    fn into(self) -> usize {
        self as usize
    }
}

#[derive(Debug)]
#[repr(usize)]
pub enum EfiWarning {
    UnknownGlyph = ffi::EFI_WARN_UNKNOWN_GLYPH, // The string contained one or more characters that the device could not render and were skipped.
    DeleteFailure = ffi::EFI_WARN_DELETE_FAILURE, // The handle was closed, but the file was not deleted.
    WriteFailure = ffi::EFI_WARN_WRITE_FAILURE, // The handle was closed, but the data to the file was not flushed properly.
    BufferTooSmall = ffi::EFI_WARN_BUFFER_TOO_SMALL, // The resulting buffer was too small, and the data was truncated to the buffer size.
    StaleData = ffi::EFI_WARN_STALE_DATA, // The data has not been updated within the timeframe set by local policy for this type of data.
    UnrecognizedWarning = <EFI_STATUS>::max_value()
}

impl From<EFI_STATUS> for EfiWarning {
    fn from(status: ffi::EFI_STATUS) -> Self {
        if ffi::IsWarning(status) { unsafe { transmute(status) } } else { EfiWarning::UnrecognizedWarning }
    }
}

#[derive(Debug, Fail)]
pub enum GeneralError {
    #[fail(display = "Failed to convert from one value to another")]
    ConversionFailed,
}
pub struct WithWarning<T> {
    pub value: T,
    pub warning: Option<EfiWarning>
}

pub type Result<T> = core::result::Result<T, EfiError>;

pub type IpAddress = ffi::EFI_IP_ADDRESS;

impl IpAddress {
    fn zero() -> Self {
        Self { Addr: [0, 0, 0, 0] }
    }
}

pub type Guid = ffi::EFI_GUID;
pub type Void = ffi::VOID;


// Fucking orphan rules
fn to_boolean(val: bool) -> ffi::BOOLEAN {
    if val { 1 } else { 0 }
}

fn from_boolean(val: ffi::BOOLEAN) -> bool {
    val != 0
}

fn to_res<T>(value: T, status: ffi::EFI_STATUS) -> Result<T> {
    match ffi::StatusType(status) {
        ffi::EFI_STATUS_TYPE::SUCCESS => Ok(value),
        _ => Err(EfiError::from(status))
    }
}

// fn to_res_with_warning<T>(value: T, status: ffi::EFI_STATUS) -> Result<WithWarning<T>> {
//     match ffi::StatusType(status) {
//         ffi::EFI_STATUS_TYPE::SUCCESS => Ok(WithWarning { value, warning: None }),
//         ffi::EFI_STATUS_TYPE::WARNING => Ok(WithWarning { value, warning: Some(EfiWarning::from(status))}),
//         ffi::EFI_STATUS_TYPE::ERROR => Err(EfiError::from(status))
//     }
// }

pub struct EfiSystemTable(pub *const EFI_SYSTEM_TABLE);

impl EfiSystemTable {
    pub fn console(&self) -> Console {
        unsafe {
            let &EfiSystemTable(table) = self;
            Console { input:  (*table).ConIn, output: (*table).ConOut }
        }
    }
}