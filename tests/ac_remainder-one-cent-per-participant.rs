//! AC: AC-remainder-one-cent-per-participant
//! Spec: SPEC-remainder-assignment

use coherence_core_db_bootstrap::split::split_evenly;

#[test]
fn validates_remainder_one_cent_per_participant() {
    for participants in 2..=9 {
        let n = participants as i64;
        let total = 100_000 - (100_000 % n) + (n - 1);
        let result = split_evenly(total, participants, 11).unwrap();

        assert_eq!(result.remainder_cents, participants - 1);
        assert_eq!(result.remainder_recipients.len(), participants - 1);

        let mut seen = result.remainder_recipients.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            participants - 1,
            "a participant must not receive two extra cents"
        );

        for allocation in &result.allocations {
            let extra = i64::from(
                result
                    .remainder_recipients
                    .contains(&allocation.participant),
            );
            assert_eq!(allocation.cents, result.base_cents + extra);
        }
    }
}
