use std::{sync::OnceLock, time::Duration};

use mach::mach_time::{mach_absolute_time, mach_timebase_info};

static TIMEBASE_INFO: OnceLock<mach_timebase_info> = OnceLock::new();

fn get_timebase_info() -> mach_timebase_info {
    *TIMEBASE_INFO.get_or_init(|| {
        let mut info: mach_timebase_info = mach_timebase_info { numer: 0, denom: 0 };
        unsafe { mach_timebase_info(&mut info) };
        info
    })
}

#[inline]
pub(in crate::engine::coreaudio) fn nanosecs_to_mach_ticks(nanos: u64) -> u64 {
    let timebase_info = get_timebase_info();
    let ticks = (nanos as u64 * timebase_info.denom as u64) / timebase_info.numer as u64;
    ticks
}

#[inline]
pub(in crate::engine::coreaudio) fn mach_ticks_to_nanosecs(ticks: u64) -> u64 {
    let timebase_info = get_timebase_info();
    let nanos = (ticks as u64 * timebase_info.numer as u64) / timebase_info.denom as u64;
    nanos
}

#[inline]
pub(in crate::engine::coreaudio) fn current_nanosecs_since_boot() -> u64 {
    let ticks = current_mach_ticks_since_boot();
    mach_ticks_to_nanosecs(ticks)
}

#[inline]
pub(in crate::engine::coreaudio) fn current_mach_ticks_since_boot() -> u64 {
    unsafe { mach_absolute_time() }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use crate::engine::coreaudio::util::{current_mach_ticks_since_boot, mach_ticks_to_nanosecs};

    #[test]
    fn test_instant_to_system_ticks() {
        let old_ticks = current_mach_ticks_since_boot();
        thread::sleep(Duration::from_millis(1));
        let new_ticks = current_mach_ticks_since_boot();
        assert_ne!(old_ticks, new_ticks);
        // It may sleep for longer than 1 millisecond, but should not be more than 1.5 millisecond.
        // ... well, it technically still can, but whatever
        assert!(mach_ticks_to_nanosecs(new_ticks - old_ticks) < 1_500_000);
    }
}
