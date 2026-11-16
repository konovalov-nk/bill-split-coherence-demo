//! AC: AC-split-equal-division-has-no-remainder
//! Spec: SPEC-bill-split

use coherence_core_db_bootstrap::money::parse_eur_to_cents;
use coherence_core_db_bootstrap::split::split_evenly;

#[test]
fn validates_split_equal_division_has_no_remainder() {
    let total = parse_eur_to_cents("999.99").unwrap();
    let result = split_evenly(total, 3, 7).unwrap();

    assert_eq!(result.remainder_cents, 0);
    assert!(result.remainder_recipients.is_empty());
    for allocation in &result.allocations {
        assert_eq!(allocation.cents, 33_333);
    }
}
