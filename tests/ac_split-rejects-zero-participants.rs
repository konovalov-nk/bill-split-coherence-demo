//! AC: AC-split-rejects-zero-participants
//! Spec: SPEC-bill-split

use coherence_core_db_bootstrap::split::{split_evenly, SplitError};

#[test]
fn validates_split_rejects_zero_participants() {
    assert_eq!(
        split_evenly(100_000, 0, 1),
        Err(SplitError::ZeroParticipants)
    );
    assert_eq!(split_evenly(0, 0, 1), Err(SplitError::ZeroParticipants));
}
