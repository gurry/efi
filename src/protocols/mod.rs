mod pxe;
mod device_path;
mod load_file;
mod loaded_image;
pub mod console;

pub use self::pxe::*;
pub use self::device_path::*;
pub use self::load_file::*;
pub use self::loaded_image::*;

use ::Guid;

pub trait Protocol {
    type FfiType;
    fn guid() -> Guid;
}