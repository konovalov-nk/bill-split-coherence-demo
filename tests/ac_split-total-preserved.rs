//! AC: AC-split-total-preserved
//! Spec: SPEC-bill-split

use coherence_core_db_bootstrap::split::split_evenly;

#[test]
fn validates_split_total_preserved() {
    for total in [0, 1, 2, 99, 100, 99_999, 100_000, 100_001, 123_456_789] {
        for participants in 1..=7 {
            for seed in 0..8 {
                let result = split_evenly(total, participants, seed).unwrap();
                let sum: i64 = result.allocations.iter().map(|a| a.cents).sum();
                assert_eq!(
                    sum, total,
                    "total={total} participants={participants} seed={seed}"
                );
                assert_eq!(result.allocations.len(), participants);
            }
        }
    }
}
