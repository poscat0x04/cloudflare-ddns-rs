#[cfg(feature = "systemd")]
use std::env::var;

#[cfg(feature = "systemd")]
use anyhow::Context;
use anyhow::Result;
#[cfg(feature = "systemd")]
use systemd::daemon::notify;

#[inline]
#[cfg(feature = "systemd")]
/// Try to detect and notify systemd
pub fn notify_startup_complete() -> Result<()> {
    if var("").is_ok() {
        notify(false, [("READY", "1")].iter())
            .context("Failed to notify systemd")?;
    }
    Ok(())
}

#[inline]
#[cfg(not(feature = "systemd"))]
/// NOOP when systemd's not enabled
pub fn notify_startup_complete() -> Result<()> { Ok(()) }