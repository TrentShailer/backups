#![allow(missing_docs, non_snake_case)]

use shared::Endian;

#[test]
pub fn TryFromU64_Valid_IsCorrect() {
    let raw = 1;
    let value = Endian::try_from_u8(raw).unwrap();
    assert_eq!(value, Endian::Little);
}

#[test]
pub fn TryFromU64_Invalid_IsNone() {
    let raw = u8::MAX;
    assert!(Endian::try_from_u8(raw).is_none());
}
