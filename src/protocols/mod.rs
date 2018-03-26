mod pxe;
mod device_path;
mod load_file;
mod loaded_image;
mod ip4;
mod tcp4;
mod simple_network;
mod managed_network;
pub mod console;

pub use self::pxe::*;
pub use self::device_path::*;
pub use self::load_file::*;
pub use self::loaded_image::*;
// pub use self::ip4::*;
// pub use self::tcp4::*;
// pub use self::simple_network::*;
// pub use self::managed_network::*;

use ::Guid;

pub trait Protocol {
    type FfiType;
    fn guid() -> Guid;
}