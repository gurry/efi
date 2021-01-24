use ffi::UINTN;
use core::time::Duration;
use crate::{system_table, Result};

pub fn sleep(dur: Duration) -> Result<()> {
    let bs = system_table().BootServices;
    let micros = (dur.as_secs() * 1000_000 + dur.subsec_micros() as u64) as UINTN; // TODO: this cast can be lossy. fix it
    unsafe { ret_on_err!(((*bs).Stall)(micros)); }
    Ok(())
}