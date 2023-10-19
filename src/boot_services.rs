use ffi::{
    boot_services::EFI_LOCATE_SEARCH_TYPE,
    EFI_HANDLE,
    EFI_GUID,
    EFI_NOT_FOUND,
    VOID,
    UINTN,
};
use crate::{Result, system_table};
use core::{ptr};
use alloc::{vec::Vec, boxed::Box};

// TODO: this guy should return an iterator to avoid allocations
pub (crate) fn locate_handles(protocol_guid: &EFI_GUID) -> Result<Vec<EFI_HANDLE>> {
    let bs = (*system_table()).BootServices;
    let mut handle_buf: *const EFI_HANDLE = ptr::null();
    let mut no_of_handles: UINTN = 0;

    let status;
    unsafe {
        status = ((*bs).LocateHandleBuffer)(EFI_LOCATE_SEARCH_TYPE::ByProtocol, protocol_guid, ptr::null() as *const VOID, &mut no_of_handles, &mut handle_buf);
    }
    if status == EFI_NOT_FOUND {
        return Ok(Vec::new()); // returning empty
    }

    ret_on_err!(status);

    let mut handles = Vec::with_capacity(no_of_handles);
    for i in 0..no_of_handles {
        unsafe { handles.push(*(handle_buf.add(i))); }
    }

    unsafe { drop(Box::from_raw(handle_buf as *mut EFI_HANDLE)); }

    Ok(handles)
}
