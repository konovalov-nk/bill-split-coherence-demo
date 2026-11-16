//! AC: AC-eur-rejects-sub-cent-amounts
//! Spec: SPEC-eur-minor-units

use coherence_core_db_bootstrap::money::{parse_eur_to_cents, MoneyError};

#[test]
fn validates_eur_rejects_sub_cent_amounts() {
    assert_eq!(
        parse_eur_to_cents("1000.001"),
        Err(MoneyError::SubCentPrecision)
    );
    assert_eq!(
        parse_eur_to_cents("1.234"),
        Err(MoneyError::SubCentPrecision)
    );
    assert_eq!(
        parse_eur_to_cents("0.0001"),
        Err(MoneyError::SubCentPrecision)
    );
    assert_eq!(parse_eur_to_cents("1000.00").unwrap(), 100_000);
}
