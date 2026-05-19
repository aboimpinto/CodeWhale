// Fix/1812: Watchdog thread that detects TUI event-loop stalls on Windows.
//
// Rapid child process exits (exec_shell) can leave the Windows console input
// handle in a non-signalling state, freezing crossterm::event::poll().  This
// watchdog monitors a heartbeat timestamp updated by the event loop on every
// iteration.  When the heartbeat is stale for more than STALL_SECONDS, it
// sets a flag that the event loop reads on its next successful poll to force
// terminal-mode recovery.
//
// Usage:
//   1. Call `tui_watchdog::start()` at TUI startup.
//   2. Call `tui_watchdog::heartbeat()` in the event loop on every iteration.
//   3. Check `tui_watchdog::recovery_needed()` and handle recovery.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Seconds of stall before the watchdog triggers recovery.
const STALL_SECONDS: u64 = 10;

static HEARTBEAT: AtomicU64 = AtomicU64::new(0);
static RECOVERY_NEEDED: AtomicBool = AtomicBool::new(false);

/// Record a heartbeat — call this once per event-loop iteration.
pub fn heartbeat() {
    let epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    HEARTBEAT.store(epoch_ms, Ordering::Release);
}

/// Returns `true` and resets the flag if the watchdog has detected a stall.
/// Call this after each `event::poll()` return to know when to force recovery.
#[must_use]
pub fn recovery_needed() -> bool {
    RECOVERY_NEEDED.swap(false, Ordering::AcqRel)
}

/// Start the watchdog thread.  Spawned once at TUI startup.
pub fn start() {
    thread::Builder::new()
        .name("tui-watchdog".into())
        .spawn(|| {
            loop {
                thread::sleep(Duration::from_secs(1));
                let last_beat = HEARTBEAT.load(Ordering::Acquire);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                if now.saturating_sub(last_beat) > STALL_SECONDS.saturating_mul(1000) {
                    RECOVERY_NEEDED.store(true, Ordering::Release);
                }
            }
        })
        .expect("spawn tui-watchdog thread");
}
