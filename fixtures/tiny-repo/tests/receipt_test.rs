//! Tests for Receipt, Replay, and Refusal capabilities
#[test]
fn test_receipt_is_emitted() {
    let r = tiny_repo::emit_receipt("scan-001");
    assert!(r.contains("Receipt"));
}

#[test]
fn test_replay_event() {
    let r = tiny_repo::replay_event(1);
    assert!(r.is_some());
}

#[test]
fn test_refusal_on_empty() {
    let r = tiny_repo::refuse("empty input");
    assert!(r.is_err());
    // Refusal is success when accurate and actionable
    assert!(r.unwrap_err().contains("Refusal"));
}

#[test]
fn test_genesis_init() {
    // Construct8 boot
    assert!(tiny_repo::genesis_init());
}
