//! Time how long a process takes to run

use std::io::Result;
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Timings {
    pub wall_time: Duration,
    pub user_time: f64,
    pub sys_time: f64,
}

/// Spawns the given command and times how long it takes to exit.
///
/// The user must ensure that no other child processes are running at the
/// same time, or else the times will be added.
///
/// On Windows the `user_time` and `sys_time` fields will be NaN.
pub fn time_cmd(cmd: Command) -> Result<(Timings, ExitStatus)> {
    #[cfg(unix)]
    let ret = time_cmd_posix(cmd)?;
    #[cfg(not(unix))]
    let ret = time_cmd_fallback(cmd)?;
    Ok(ret)
}

#[cfg(not(unix))]
fn time_cmd_fallback(mut cmd: Command) -> Result<(Timings, ExitStatus)> {
    let ts = Instant::now();
    let status = cmd.spawn()?.wait()?;
    let d = ts.elapsed();
    Ok((
        Timings {
            wall_time: d,
            user_time: std::f64::NAN,
            sys_time: std::f64::NAN,
        },
        status,
    ))
}

#[cfg(unix)]
fn time_cmd_posix(mut cmd: Command) -> Result<(Timings, ExitStatus)> {
    // times(2) and sysconf(2) are both POSIX
    let mut tms_before = libc::tms {
        tms_utime: 0,
        tms_stime: 0,
        tms_cutime: 0,
        tms_cstime: 0,
    };
    let mut tms_after = tms_before;

    unsafe { libc::times(&mut tms_before as *mut libc::tms) };
    let ts = Instant::now();
    let status = cmd.spawn()?.wait()?;
    let d = ts.elapsed();
    unsafe { libc::times(&mut tms_after as *mut libc::tms) };

    let ticks_per_sec = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
    let utime = (tms_after.tms_cutime - tms_before.tms_cutime) as f64 / ticks_per_sec;
    let stime = (tms_after.tms_cstime - tms_before.tms_cstime) as f64 / ticks_per_sec;

    Ok((
        Timings {
            wall_time: d,
            user_time: utime,
            sys_time: stime,
        },
        status,
    ))
}
