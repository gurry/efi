// TODO: Write a proc macro called derive(TupleWrapper) which automaticlly impls Wrapper trait for any tuple struct wrapping types
use core;

pub trait Wrapper {
    type Inner;
    fn inner_ptr(&self) -> *const Self::Inner;
}

pub fn to_ptr<'a, W: Wrapper>(value: Option<&'a W>) -> *const W::Inner {
    value.map_or(core::ptr::null(), |v| v.inner_ptr())
}

pub fn to_ptr_mut<'a, W: Wrapper>(value: Option<&'a mut W>) -> *mut W::Inner {
    value.map_or(core::ptr::null::<W::Inner>() as *mut W::Inner, |v| v.inner_ptr() as *mut W::Inner)
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
