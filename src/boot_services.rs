use ffi::{
    boot_services::EFI_LOCATE_SEARCH_TYPE,
    EFI_HANDLE,
    EFI_GUID,
    EFI_NOT_FOUND,
    VOID,
    UINTN,
};
use ::{Result, system_table, boxed::EfiBox};
use core::{ptr};
use alloc::vec::Vec;

// TODO: this guy should return an iterator to avoid allocations
pub (crate) fn locate_handles(protocol_guid: &EFI_GUID) -> Result<Vec<EFI_HANDLE>> {
    let bs = (*system_table()).BootServices;
    let mut handle_buf: *const EFI_HANDLE = ptr::null();
    let mut no_of_handles: UINTN = 0;
    unsafe {
        let status = ((*bs).LocateHandleBuffer)(EFI_LOCATE_SEARCH_TYPE::ByProtocol, protocol_guid, ptr::null() as *const VOID, &mut no_of_handles, &mut handle_buf);
        if status == EFI_NOT_FOUND {
            return Ok(Vec::new()); // returning empty
        }

        ret_on_err!(status);

        let handle_box = EfiBox::from_raw(handle_buf as *mut EFI_HANDLE); // Just so that we deallocat this buffer when we go out of scope

        let mut handles = Vec::with_capacity(no_of_handles);
        for i in 0..no_of_handles {
            handles.push(*(handle_box.as_raw().add(i)));
        }

        Ok(handles)
    }
}
