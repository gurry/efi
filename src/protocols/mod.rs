mod pxe;
mod device_path;
mod load_file;
pub mod console;

pub use self::pxe::*;
pub use self::device_path::*;
pub use self::load_file::*;

use ::Guid;

pub trait Protocol {
    type FfiType;
    fn guid() -> Guid;
}