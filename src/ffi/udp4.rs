use ffi::UINTN;
use core::mem;

pub const EFI_NETWORK_UNREACHABLE: UINTN = with_high_bit_set!(100);
pub const EFI_HOST_UNREACHABLE: UINTN = with_high_bit_set!(101) ;
pub const EFI_PROTOCOL_UNREACHABLE: UINTN = with_high_bit_set!(102);
pub const EFI_PORT_UNREACHABLE: UINTN = with_high_bit_set!(103);