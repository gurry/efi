// TODO: Write a proc macro called derive(TupleWrapper) which automaticlly impls Wrapper trait for any tuple struct wrapping types

pub trait Wrapper {
    type Inner;
    fn inner_ptr(&self) -> *const Self::Inner;
}

// impl<W: Wrap, I> From<Option<&W>> for *const I {
//     fn from(value: Option<&W>) -> *const I {
//         value.map_or(ptr::null(), |v| unsafe { mem::transmute(v) });
//     }
// } 

// macro_rules! tuple_struct_wrapper {
//     ($wrapper: ty, $inner: ty) => {
//         struct $wrapper($inner);

//         impl Wrapper for $wrapper {
//             type Inner = %inner;
//             fn as_inner_ptr(&self) -> *const Inner {
//                 use core::mem::transmute;
//                 transmute(&self.0);
//             }
//         }
//     };
// }