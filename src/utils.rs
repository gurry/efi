// TODO: Write a proc macro called derive(TupleWrapper) which automaticlly impls Wrapper trait for any tuple struct wrapping types
use core::{self, mem};

pub trait Wrapper {
    type Inner;
    fn inner_ptr(&self) -> *const Self::Inner;
}

pub fn to_ptr<'a, W: Wrapper>(value: Option<&'a W>) -> *const W::Inner {
    value.map_or(core::ptr::null(), |v| v.inner_ptr())
}


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

macro_rules! impl_protocol {
    ($wrapper: ty, $inner: ty, $guid:tt) => {
        impl Protocol for $wrapper {
            type FfiType = $inner;
            fn guid() -> Guid {
                $guid
            }
        }
    }
}

macro_rules! ret_on_err {
    ($e:expr) => {
        let status: ::ffi::EFI_STATUS = $e;
        if !IsSuccess(status) {
            return Err(EfiError::from(status));
        }
    }
}