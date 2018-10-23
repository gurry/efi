// TODO: Write a proc macro called derive(TupleWrapper) which automaticlly impls Wrapper trait for any tuple struct wrapping types
use ffi::CHAR16;
use core::{self, mem, slice};

pub trait Wrapper {
    type Inner;
    fn inner_ptr(&self) -> *const Self::Inner;
}

pub fn to_ptr<'a, W: Wrapper>(value: Option<&'a W>) -> *const W::Inner {
    value.map_or(core::ptr::null(), |v| v.inner_ptr())
}


// TODO: In rust an Option<*T> is represented the same way as *T
// So we can use Options directly instead of using this method
pub fn to_opt<'a, P, R>(ptr: *const P) -> Option<&'a R> {
    unsafe { ptr.as_ref().map(|p| mem::transmute(p)) }  
}

macro_rules! impl_wrapper {
    ($wrapper: ty, $inner: ty) => {
        impl ::utils::Wrapper for $wrapper {
            type Inner = $inner;
            fn inner_ptr(&self) -> *const Self::Inner {
                use core::mem::transmute;
                unsafe { transmute(&self.0) }
            }
        }
    };
}

macro_rules! ret_on_err {
    ($e:expr) => {
        let status: ::ffi::EFI_STATUS = $e;
        if !$crate::ffi::IsSuccess(status) {
            return Err($crate::EfiError::from(status));
        }
    }
}

pub unsafe fn as_slice<'a>(s: *const CHAR16) -> &'a [CHAR16] {
    let mut len = 0;
    let mut temp = s;
    while  *temp != 0 {
        len += 1;
        temp = temp.offset(1);
    }

    slice::from_raw_parts(s, len)
}

#[derive(Debug)]
pub struct NullTerminatedAsciiStr<'a> {
    buffer: &'a [u8]
}

impl<'a> NullTerminatedAsciiStr<'a> {
    pub fn new(buffer: &[u8]) -> NullTerminatedAsciiStr {
        assert!(buffer.len() >= 1);
        assert!(buffer[buffer.len() - 1] == 0);
        NullTerminatedAsciiStr { buffer: buffer }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr() as *const u8
    }
}