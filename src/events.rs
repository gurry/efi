use ffi::{
    UINT64,
    EFI_SUCCESS,
    EFI_NOT_READY,
    EFI_EVENT,
    boot_services::{
        EVT_NOTIFY_WAIT,
        EVT_NOTIFY_SIGNAL,
        EVT_TIMER,
        EFI_TPL,
        TPL_CALLBACK,
        TPL_NOTIFY,
        // TPL_HIGH_LEVEL,
        EFI_TIMER_DELAY,
    },
};

use core::{ptr, time::Duration};
use crate::{system_table, Result};

pub trait Signal {
    fn signal(&mut self) -> Result<()>;
}

pub trait Wait {
    fn wait(&self) -> Result<()>;
    fn is_signaled(&self) ->  Result<bool>;
}

pub trait AsRawEvt {
    unsafe fn as_raw(&self) -> EFI_EVENT;
}

#[repr(u32)]
pub enum NotifyType {
    Wait = EVT_NOTIFY_WAIT,
    Signal = EVT_NOTIFY_SIGNAL,
}

#[repr(usize)]
pub enum EventTpl {
    Callback = TPL_CALLBACK,
    Notify = TPL_NOTIFY,
    // HighLevel = TPL_HIGH_LEVEL, // TODo: Should we expose HighLevel or not? It can slow this system down if used irresponsibly.
}

// SOME COMMENTS ON DIFFERETNT EVENT TYPES (to help us with the design in future):
// - EVT_NOTIFY_* means the event has an associated callback to call. If there's no EVT_NOTIFY_* attribute then it means there's no callback.
// - Of the two EVT_NOTIFY_* attributes, EVT_NOTIFY_SIGNAL means the callback will be called only when the event is signaled.
//      and EVT_NOTIFY_WAIT means the callback will be called repeatedly UNTIL the event is signaled.
// - EVT_NOTIFY_SIGNAL also means you can't call CheckEvent() or WaitForEvent() on the event. 
//      EVT_NOTIFY_WAIT means you can call CheckEvent() and WaitForEvent() on the event.
// - The above differences between EVT_NOTIFY_SIGNAL and EVT_NOTIFY_WAIT are the reason why these attributes are mutually
//      exclusive and hence can't be specified together.
// - EVT_TIMER is an attribute that is orthogogal to the above two and can be specified in combination with them. What is means 
//      basically is that the event is a timer and therefore you can call SetTimer() on this event. You can't call this function 
//      on an event if EVT_TIMER attribute is not present.


// TODO: Disabled until we figured out a better design. Enable them back
// extern "win64" fn common_notify_func<F: FnMut()>(_event: EFI_EVENT, context: *const VOID) -> EFI_STATUS {
//     if !context.is_null() {
//         let closure: *mut F = unsafe { mem::transmute(context) }; // Safe to make this transmute because we know this is the pointer to the closure
//         unsafe { (*closure)(); }
//     }
//     EFI_SUCCESS
// }

// struct Event<F: FnMut()>
// {
//     inner: EFI_EVENT,
//     _notify_func: Option<Box<F>>, // On heap 'cause we pass its underlying raw ptr to notify func as context. Hence needs to be stable ptr.
// }

// impl<F: FnMut()> Event<F> {
//     fn create<N: Into<Option<F>>>(notify_flags: UINT32, tpl: EventTpl, notify_func: N) -> Result<Self> {
//         let bs = system_table().BootServices;
//         let notify_func: Option<F> = notify_func.into();

//         let raw_func_ptr =  if let Some(notify_func) = notify_func {
//             let boxed_notify_func = Box::new(notify_func);
//             Box::into_raw(boxed_notify_func)
//         } else {
//             ptr::null_mut()
//         };

//         let mut event: EFI_EVENT = ptr::null();
//         unsafe {
//             ret_on_err!(((*bs).CreateEvent)(notify_flags, tpl as EFI_TPL, common_notify_func::<F>, raw_func_ptr as *const VOID, &mut event));
//         }

//         let notify_func_opt = if !raw_func_ptr.is_null() {
//             Some(unsafe { Box::from_raw(raw_func_ptr) })
//         } else {
//             None
//         };

//         Ok(Self { inner: event, _notify_func: notify_func_opt })
//     }

//     fn wait(&self) -> Result<()> {
//         let bs = system_table().BootServices;
//         unsafe {
//             let mut signaled_index = 0;
//             ret_on_err!(((*bs).WaitForEvent)(1, &self.inner, &mut signaled_index));
//         }

//         Ok(())
//     }

//     fn signal(&mut self) -> Result<()> {
//         let bs = system_table().BootServices;
//         unsafe { ret_on_err!(((*bs).SignalEvent)(self.inner)); }
//         Ok(())
//     }

//     fn is_signaled(&self) ->  Result<bool> {
//         let bs = system_table().BootServices;
//         let status = unsafe { ((*bs).CheckEvent)(self.inner) };
//         match status {
//             EFI_SUCCESS => Ok(true),
//             EFI_NOT_READY=> Ok(false),
//             s => Err(s.into())
//         }
//     }

//     #[inline]
//     fn as_raw(&self) -> EFI_EVENT {
//         self.inner
//     }
// }

// impl<F: FnMut()> Drop for Event<F> {
//     fn drop(&mut self) {
//         let bs = system_table().BootServices;
//         unsafe {
//             ((*bs).CloseEvent)(self.inner); // Can't do a fucking thing if it returns failure
//         }
//     }
// }

// pub struct NotifySignalEvent<F: FnMut()>(Event<F>);

// impl<F: FnMut()> NotifySignalEvent<F> {
//     pub fn create<N: Into<Option<F>>>(tpl: EventTpl, notify_func: N) -> Result<Self> {
//         let inner = Event::create(NotifyType::Signal as UINT32, tpl, notify_func)?;
//         Ok(NotifySignalEvent(inner))
//     }
// }

// impl<F: FnMut()> Signal for NotifySignalEvent<F> {
//     #[inline]
//     fn signal(&mut self) -> Result<()> {
//         self.0.signal()
//     }
// }

// impl<F: FnMut()> AsRawEvt for NotifySignalEvent<F> {
//     #[inline]
//     unsafe fn as_raw(&self) -> EFI_EVENT {
//         self.0.as_raw()
//     }
// }

// pub struct NotifyWaitEvent<F: FnMut()>(Event<F>);

// impl<F: FnMut()> NotifyWaitEvent<F> {
//     pub fn create<N: Into<Option<F>>>(tpl: EventTpl, notify_func: N) -> Result<Self> {
//         let inner = Event::create(NotifyType::Wait as UINT32, tpl, notify_func)?;
//         Ok(NotifyWaitEvent(inner))
//     }
// }

// impl<F: FnMut()> Signal for NotifyWaitEvent<F> {
//     #[inline]
//     fn signal(&mut self) -> Result<()> {
//         self.0.signal()
//     }
// }

// impl<F: FnMut()> Wait for NotifyWaitEvent<F> {
//     #[inline]
//     fn wait(&self) -> Result<()> {
//         self.0.wait()
//     }

//     #[inline]
//     fn is_signaled(&self) ->  Result<bool> {
//         self.0.is_signaled()
//     }
// }

// impl<F: FnMut()> AsRawEvt for NotifyWaitEvent<F> {
//     #[inline]
//     unsafe fn as_raw(&self) -> EFI_EVENT {
//         self.0.as_raw()
//     }
// }

pub enum TimerSchedule {
    Relative,
    Periodic
}

impl TimerSchedule {
    fn as_raw(&self) -> EFI_TIMER_DELAY {
        match self {
            TimerSchedule::Relative => EFI_TIMER_DELAY::TimerRelative,
            TimerSchedule::Periodic => EFI_TIMER_DELAY::TimerPeriodic,
        }
    }
}
pub enum TimerState {
    Active,
    Inactive,
}

pub struct Timer(EFI_EVENT);

impl Timer {
    pub fn create(interval: Duration, schedule: TimerSchedule, state: TimerState, tpl: EventTpl) -> Result<Self> {
        let bs = system_table().BootServices;
        let mut event: EFI_EVENT = ptr::null();
        unsafe {
            ret_on_err!(((*bs).CreateEvent)(EVT_TIMER, tpl as EFI_TPL, None, ptr::null(), &mut event));
        }

        let mut timer = Timer(event);
        match state {
            TimerState::Active => timer.set(interval, schedule)?,
            _ => (),
        };

        Ok(timer)
    }

    pub fn set(&mut self, interval: Duration, schedule: TimerSchedule) -> Result<()> {
        let bs = system_table().BootServices;
        unsafe {
            ret_on_err!(((*bs).SetTimer)(self.0, schedule.as_raw(), as_100ns_units(&interval)));
        }

        Ok(())
    }

    pub fn cancel(&mut self) -> Result<()> {
        let bs = system_table().BootServices;
        unsafe {
            ret_on_err!(((*bs).SetTimer)(self.0, EFI_TIMER_DELAY::TimerCancel, 0));
        }
        
        Ok(())
    }

}

impl Drop for Timer {
    fn drop(&mut self) {
        let bs = system_table().BootServices;
        unsafe {
            ((*bs).CloseEvent)(self.0); // Can't do a fucking thing if it returns failure
        }
    }
}

impl Wait for Timer {
    fn wait(&self) -> Result<()> {
        let bs = system_table().BootServices;
        unsafe {
            let mut signaled_index = 0;
            ret_on_err!(((*bs).WaitForEvent)(1, &self.0, &mut signaled_index));
        }

        Ok(())
    }

    fn is_signaled(&self) ->  Result<bool> {
        let bs = system_table().BootServices;
        let status = unsafe { ((*bs).CheckEvent)(self.0) };
        match status {
            EFI_SUCCESS => Ok(true),
            EFI_NOT_READY=> Ok(false),
            s => Err(s.into())
        }
    }
}

impl AsRawEvt for Timer {
    #[inline]
    unsafe fn as_raw(&self) -> EFI_EVENT {
        self.0
    }
}


// TODO: Disabled until we figured out a better design. Enable them back
// pub struct NotifySignalTimer<F: FnMut()>(Timer<F>);

// impl<F: FnMut()> NotifySignalTimer<F> {
//     pub fn create<N: Into<Option<F>>>(tpl: EventTpl, notify_func: N) -> Result<Self> {
//         let inner = Timer::create(NotifyType::Signal, tpl, notify_func)?;
//         Ok(NotifySignalTimer(inner))
//     }

//     #[inline]
//     pub fn set(&mut self, interval: Duration, schedule: TimerSchedule) -> Result<()> {
//         self.0.set(interval, schedule)
//     }

//     #[inline]
//     pub fn cancel(&mut self) -> Result<()> {
//         self.0.cancel()
//     }
// }

// impl<F: FnMut()> AsRawEvt for NotifySignalTimer<F> {
//     #[inline]
//     unsafe fn as_raw(&self) -> EFI_EVENT {
//         self.0.as_raw()
//     }
// }

// pub struct NotifyWaitTimer<F: FnMut()>(Timer<F>);

// impl<F: FnMut()> NotifyWaitTimer<F> {
//     pub fn create<N: Into<Option<F>>>(tpl: EventTpl, notify_func: N) -> Result<Self> {
//         let inner = Timer::create(NotifyType::Wait, tpl, notify_func)?;
//         Ok(NotifyWaitTimer(inner))
//     }

//     #[inline]
//     pub fn set(&mut self, interval: Duration, schedule: TimerSchedule) -> Result<()> {
//         self.0.set(interval, schedule)
//     }

//     #[inline]
//     pub fn cancel(&mut self) -> Result<()> {
//         self.0.cancel()
//     }
// }

// impl<F: FnMut()> Wait for NotifyWaitTimer<F> {
//     #[inline]
//     fn wait(&self) -> Result<()> {
//         self.0.wait()
//     }
    
//     #[inline]
//     fn is_signaled(&self) ->  Result<bool> {
//         self.0.is_signaled()
//     }
// }

// impl<F: FnMut()> AsRawEvt for NotifyWaitTimer<F> {
//     #[inline]
//     unsafe fn as_raw(&self) -> EFI_EVENT {
//         self.0.as_raw()
//     }
// }

fn as_100ns_units(dur: &Duration) -> UINT64 {
    const T_100NS_UNITS_IN_A_SEC: UINT64 = 10_000_000;
    const T_100NS_UNITS_IN_A_MICRO: UINT64  = 10;
    (dur.as_secs()* T_100NS_UNITS_IN_A_SEC) + (dur.subsec_micros() as u64 * T_100NS_UNITS_IN_A_MICRO)
}