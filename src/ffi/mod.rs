// This API implements UEFI spec version 2.4

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

mod base;
pub mod pxe;
pub mod load_file;
pub mod device_path;
pub mod console;
pub mod boot_services;

pub use self::base::*;
use ffi::boot_services::EFI_BOOT_SERVICES;
use ffi::console::{EFI_SIMPLE_TEXT_INPUT_PROTOCOL, EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL};

#[repr(C)]
pub struct EFI_SYSTEM_TABLE {
    pub Hdr : EFI_TABLE_HEADER,
    pub FirmwareVendor : *const u16,
    pub FirmwareRevision : u32,
    pub ConsoleInHandle : EFI_HANDLE,
    pub ConIn : *const EFI_SIMPLE_TEXT_INPUT_PROTOCOL,
    pub ConsoleOutHandle : EFI_HANDLE,
    pub ConOut : *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    pub ConsoleErrorHandle : EFI_HANDLE,
    pub StdErr : *const EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL,
    pub RuntimeServices : *const EFI_RUNTIME_SERVICES,
    pub BootServices : *const EFI_BOOT_SERVICES,
    pub NumberOfTableEntries : usize,
    pub ConfigurationTable : *const EFI_CONFIGURATION_TABLE
}


pub struct EFI_RUNTIME_SERVICES;

#[repr(C)]
pub struct EFI_CONFIGURATION_TABLE {
    pub VendorGuid : base::EFI_GUID,
    pub VendorTable : *const ()
}