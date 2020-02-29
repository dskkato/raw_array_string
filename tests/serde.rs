#![cfg(feature = "serde")]
extern crate raw_array_string;
extern crate serde_test;

use raw_array_string::RawArrayString;

use serde_test::{assert_de_tokens_error, assert_tokens, Token};

#[test]
fn test_ser_de_empty() {
    let string = RawArrayString::<[u8; 0]>::new();

    assert_tokens(&string, &[Token::Str("")]);
}

#[test]
fn test_ser_de() {
    let string = RawArrayString::<[u8; 9]>::from("1234 abcd")
        .expect("expected exact specified capacity to be enough");

    assert_tokens(&string, &[Token::Str("1234 abcd")]);
}

#[test]
fn test_de_too_large() {
    assert_de_tokens_error::<RawArrayString<[u8; 2]>>(
        &[Token::Str("afd")],
        "invalid length 3, expected a string no more than 2 bytes long",
    );
}
