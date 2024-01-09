#![allow(dead_code)]
/// Wrapper for base64 crate
/// https://github.com/marshallpierce/rust-base64/issues/213
use base64::{engine, Engine as _};

pub use base64::DecodeError;

pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
    engine::general_purpose::STANDARD.decode(input)
}

pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    engine::general_purpose::STANDARD.encode(input)
}

pub fn encode_urlsafe_no_pad<T: AsRef<[u8]>>(input: T) -> String {
    engine::general_purpose::URL_SAFE_NO_PAD.encode(input)
}