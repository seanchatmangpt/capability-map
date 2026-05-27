//! tiny-repo lib.rs — fixture for cpmp capability detection
//!
//! Contains references to: Receipt, Replay, Refusal, Genesis, Construct8

/// Emit a Receipt for a completed operation.
pub fn emit_receipt(id: &str) -> String {
    format!("Receipt::{}", id)
}

/// Replay an event from the log.
pub fn replay_event(seq: u64) -> Option<String> {
    if seq == 0 { return None; }
    Some(format!("Replay::{}", seq))
}

/// Refuse an invalid request.
pub fn refuse(reason: &str) -> Result<(), String> {
    Err(format!("Refusal: {}", reason))
}

/// Genesis initializer — the lawful construction kernel entry.
pub fn genesis_init() -> bool {
    // Construct8 routes to w1 by default
    true
}
