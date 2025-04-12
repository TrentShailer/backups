#![allow(missing_docs)]

use shared::Cadance;

#[test]
pub fn valid_cadance() {
    let value = 3;
    let cadance = Cadance::try_from_u64(value).unwrap();
    assert_eq!(cadance, Cadance::Monthly);
}

#[test]
pub fn invalid_value() {
    let value = u64::MAX;
    assert!(Cadance::try_from_u64(value).is_none());
}
