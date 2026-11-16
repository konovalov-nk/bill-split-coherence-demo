//! AC: AC-remainder-seeded-choice-reproducible
//! Spec: SPEC-remainder-assignment

use std::collections::BTreeSet;

use coherence_core_db_bootstrap::split::split_evenly;

#[test]
fn validates_remainder_seeded_choice_reproducible() {
    let first = split_evenly(100_000, 3, 7).unwrap();
    let second = split_evenly(100_000, 3, 7).unwrap();
    assert_eq!(first, second);

    let mut observed = BTreeSet::new();
    for seed in 0..32 {
        let result = split_evenly(100_000, 3, seed).unwrap();
        observed.insert(result.remainder_recipients);
    }
    assert!(
        observed.len() >= 2,
        "seed must be able to select different participants: {observed:?}"
    );
}
