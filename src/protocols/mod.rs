mod pxe;
pub mod console;

pub use self::pxe::*;

use ::Guid;

pub trait Protocol {
    type FfiType;
    fn guid() -> Guid;
}