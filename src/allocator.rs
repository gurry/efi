
use system_table;
use alloc::allocator::{Alloc, AllocErr, Layout};
use ffi::{EFI_SUCCESS, EFI_OUT_OF_RESOURCES, VOID, boot_services::EFI_MEMORY_TYPE};
use core::ptr;

pub struct EfiAllocator;

unsafe impl<'a> Alloc for &'a EfiAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        if layout.size() == 0 {  // Zero sized requests can be valid as per Rust's documentation, but we don't want to support it
            return Err(AllocErr::Unsupported { details: "Zero sized alloc request"});
        }

        if layout.align() % 2 != 0 && layout.align() != 1 { // Just in case some fucker asks for an odd-number alignment -- except if it's 1 which is fine
            return Err(AllocErr::Unsupported { details: "Odd-number alignment alloc request"});
        }

        // TODO: Add support for alignment greater than 8. 
        // UEFI always allocates to 8-byte aligntment. So we're fine if align() says 8 or less.
        // If align() asks for something greater than 8 then we can handle that by rounding up here 
        // and by doing the converse calculation in dealloc() below. This is yet to be implemented.
        if layout.align() > 8  {
            return Err(AllocErr::Unsupported { details: "Request with alignment greater than 8"});
        }

        let mut ptr = ptr::null() as *const VOID;
        let status = ((*system_table().BootServices).AllocatePool)(EFI_MEMORY_TYPE::EfiLoaderData, layout.size(), &mut ptr);
        match status {
            EFI_SUCCESS => Ok(ptr as *mut u8),
            EFI_OUT_OF_RESOURCES => Err(AllocErr::Exhausted { request: layout }),
            _ => Err(AllocErr::Unsupported { details: "UEFI AllocatePool returned an error"}),
        }
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, _layout: Layout) {
        // TODO: As mentioned above, stop ignoring layout::align() here
        let status = ((*system_table().BootServices).FreePool)(ptr as *const VOID);

        if status != EFI_SUCCESS {
            panic!("UEFI FreePool returned an error");
        }
    }
}