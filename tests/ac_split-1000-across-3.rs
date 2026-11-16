//! AC: AC-split-1000-across-3
//! Spec: SPEC-bill-split

use coherence_core_db_bootstrap::money::parse_eur_to_cents;
use coherence_core_db_bootstrap::split::split_evenly;

#[test]
fn validates_split_1000_across_3() {
    let total = parse_eur_to_cents("1000.00").unwrap();
    let result = split_evenly(total, 3, 42).unwrap();

    let mut shares: Vec<i64> = result.allocations.iter().map(|a| a.cents).collect();
    shares.sort_unstable();

    assert_eq!(shares, vec![33_333, 33_333, 33_334]);
    assert_eq!(result.total_cents, 100_000);
    assert_eq!(result.base_cents, 33_333);
    assert_eq!(result.remainder_cents, 1);
    assert_eq!(result.remainder_recipients.len(), 1);
}
