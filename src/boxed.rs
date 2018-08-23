use core::{
    mem, fmt, cmp::Ordering, 
    ops::{DerefMut, Deref},
    borrow, ptr::{self, Unique}
};
use ffi::{
    boot_services::EFI_MEMORY_TYPE,
    EFI_SUCCESS,
    VOID,
};
use {system_table, Result};

pub struct EfiBox<T>(Unique<T>);

impl<T> EfiBox<T> {
    #[inline]
    pub unsafe fn allocate(size: usize) -> Result<Self> {
        let mut ptr = ptr::null() as *const VOID;
        let status = ((*system_table().BootServices).AllocatePool)(EFI_MEMORY_TYPE::EfiLoaderData, size, &mut ptr);
        match status {
            EFI_SUCCESS => {
                let unique = Unique::new_unchecked(ptr as *mut T);
                Ok(EfiBox(unique))
            },
            e => Err(e.into()),
        }
    }

    #[inline]
    pub unsafe fn from_raw(raw: *mut T) -> Self {
        EfiBox(Unique::new_unchecked(raw))
    }

    #[inline]
    pub fn into_raw(self) -> *mut T {
        let raw = self.as_raw();
        mem::forget(self);
        raw
    }

    #[inline]
    pub fn as_raw(&self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T> Drop for EfiBox<T> {
    fn drop(&mut self) {
        unsafe { ((*system_table().BootServices).FreePool)(self.as_raw() as *const VOID) }; // No need to check status. Can't do anything if it fails.
    }
}

impl<T: PartialEq> PartialEq for EfiBox<T> {
    #[inline]
    fn eq(&self, other: &EfiBox<T>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
    #[inline]
    fn ne(&self, other: &EfiBox<T>) -> bool {
        PartialEq::ne(&**self, &**other)
    }
}

impl<T: PartialOrd> PartialOrd for EfiBox<T> {
    #[inline]
    fn partial_cmp(&self, other: &EfiBox<T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &EfiBox<T>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    #[inline]
    fn le(&self, other: &EfiBox<T>) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    #[inline]
    fn ge(&self, other: &EfiBox<T>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    #[inline]
    fn gt(&self, other: &EfiBox<T>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: Ord> Ord for EfiBox<T> {
    #[inline]
    fn cmp(&self, other: &EfiBox<T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: Eq> Eq for EfiBox<T> {}

impl<T: fmt::Display> fmt::Display for EfiBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for EfiBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T> fmt::Pointer for EfiBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // It's not possible to extract the inner Uniq directly from the EfiBox,
        // instead we cast it to a *const which aliases the Unique
        let ptr: *const T = &**self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

impl<T> Deref for EfiBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.as_raw() }
    }
}

impl<T> DerefMut for EfiBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.as_raw() }
    }
}

impl<T> borrow::Borrow<T> for EfiBox<T> {
    fn borrow(&self) -> &T {
        &**self
    }
}

impl<T> borrow::BorrowMut<T> for EfiBox<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T> AsRef<T> for EfiBox<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T> AsMut<T> for EfiBox<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}