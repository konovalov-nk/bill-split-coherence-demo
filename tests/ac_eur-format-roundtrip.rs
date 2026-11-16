//! AC: AC-eur-format-roundtrip
//! Spec: SPEC-eur-minor-units

use coherence_core_db_bootstrap::money::{format_cents, parse_eur_to_cents};

#[test]
fn validates_eur_format_roundtrip() {
    for cents in [0, 1, 99, 100, 33_333, 33_334, 100_000, 123_456_789] {
        let text = format_cents(cents);
        assert_eq!(parse_eur_to_cents(&text).unwrap(), cents, "cents={cents}");
    }
    assert_eq!(format_cents(33_334), "333.34");
    assert_eq!(format_cents(1), "0.01");
    assert_eq!(format_cents(100_000), "1000.00");
}
