
use crate::system_table;
use ffi::{EFI_SUCCESS, VOID, boot_services::EFI_MEMORY_TYPE};
use core::{
    ptr,
    alloc::{GlobalAlloc, Layout},
};

pub struct EfiAllocator;

// unsafe impl<'a> Alloc for &'a EfiAllocator {
// unsafe impl<'a> Alloc for &'a EfiAllocator {
//     unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
// NonNull::new_unchcked()
//     }

//     unsafe fn dealloc(&mut self, ptr: NonNull<u8>, _layout: Layout) {
// }

unsafe impl GlobalAlloc for EfiAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    // unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        // TODO: Add support for alignment greater than 8. 
        if layout.size() == 0 || // Zero sized requests can be valid as per Rust's documentation, but we don't want to support it
            layout.align() % 2 != 0 && layout.align() != 1 ||  // Just in case some fucker asks for an odd-number alignment -- except if it's 1 which is fine
            // UEFI always allocates to 8-byte aligntment. So we're fine if align() says 8 or less.
            // If align() asks for something greater than 8 then we can handle that by rounding up here 
            // and by doing the converse calculation in dealloc() below. This is yet to be implemented.
            layout.align() > 8 {
            return ptr::null_mut();
        }

        let mut ptr = ptr::null() as *const VOID;
        let status = ((*system_table().BootServices).AllocatePool)(EFI_MEMORY_TYPE::EfiLoaderData, layout.size(), &mut ptr);
        match status {
            EFI_SUCCESS => ptr as *mut u8,
            _ => ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // TODO: As mentioned above, stop ignoring layout::align() here
        let status = ((*system_table().BootServices).FreePool)(ptr as *const VOID);

        if status != EFI_SUCCESS {
            panic!("UEFI FreePool returned an error");
        }
    }
}