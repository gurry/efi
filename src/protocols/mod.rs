mod pxe;
pub mod console;

pub use self::pxe::*;

use ::Guid;
use core::convert::From;

pub trait Protocol: From<*const <Self as Protocol>::FfiType> {
    type FfiType;
    fn guid() -> Guid;
}