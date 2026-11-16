//! AC: AC-eur-parses-decimal-to-cents
//! Spec: SPEC-eur-minor-units

use coherence_core_db_bootstrap::money::parse_eur_to_cents;

#[test]
fn validates_eur_parses_decimal_to_cents() {
    assert_eq!(parse_eur_to_cents("1000.00").unwrap(), 100_000);
    assert_eq!(parse_eur_to_cents("333.33").unwrap(), 33_333);
    assert_eq!(parse_eur_to_cents("0.01").unwrap(), 1);
    assert_eq!(parse_eur_to_cents("1.2").unwrap(), 120);
    assert_eq!(parse_eur_to_cents("1000").unwrap(), 100_000);
    assert_eq!(parse_eur_to_cents("1000.00 €").unwrap(), 100_000);
}
