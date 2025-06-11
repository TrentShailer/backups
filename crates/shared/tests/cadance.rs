#![allow(missing_docs, non_snake_case)]

use shared::Cadence;

#[test]
pub fn TryFromU64_Valid_IsCorrect() {
    let value = 3;
    let cadence = Cadence::try_from_u64(value).unwrap();
    assert_eq!(cadence, Cadence::Monthly);
}

#[test]
pub fn TryFromU64_Invalid_IsNone() {
    let value = u64::MAX;
    assert!(Cadence::try_from_u64(value).is_none());
}
