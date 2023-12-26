use rsa::{
    pkcs1::{
        DecodeRsaPrivateKey,
        DecodeRsaPublicKey,
        EncodeRsaPrivateKey,
        EncodeRsaPublicKey,
    },
    pkcs8::{
        DecodePrivateKey,
        DecodePublicKey,
        EncodePrivateKey,
        EncodePublicKey,
        LineEnding,
    },
    pkcs1v15::{Signature, SigningKey, VerifyingKey},
    signature::{
        SignatureEncoding,
        Signer,
        Verifier,
    },
};
use sha2::Sha256;

pub use rsa::{RsaPrivateKey, RsaPublicKey};
pub type RsaError = rsa::errors::Error;

pub fn generate_rsa_key() -> Result<RsaPrivateKey, RsaError> {
    let mut rng = rand::rngs::OsRng;
    let bits = 2048;
    RsaPrivateKey::new(&mut rng, bits)
}

#[cfg(feature = "test-utils")]
pub fn generate_weak_rsa_key() -> Result<RsaPrivateKey, RsaError> {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let bits = 512;
    RsaPrivateKey::new(&mut rng, bits)
}

#[derive(thiserror::Error, Debug)]
pub enum RsaSerializationError {
    #[error(transparent)]
    Pkcs1Error(#[from] rsa::pkcs1::Error),

    #[error(transparent)]
    Pkcs8Error(#[from] rsa::pkcs8::Error),

    #[error(transparent)]
    PemError(#[from] pem::PemError),
}

pub fn rsa_private_key_to_pkcs1_der(
    private_key: &RsaPrivateKey,
) -> Result<Vec<u8>, RsaSerializationError> {
    let bytes = private_key.to_pkcs1_der()?.as_bytes().to_vec();
    Ok(bytes)
}

pub fn rsa_private_key_from_pkcs1_der(
    bytes: &[u8],
) -> Result<RsaPrivateKey, RsaSerializationError> {
    let private_key = RsaPrivateKey::from_pkcs1_der(bytes)?;
    Ok(private_key)
}

pub fn rsa_public_key_to_pkcs1_der(
    public_key: &RsaPublicKey,
) -> Result<Vec<u8>, RsaSerializationError> {
    let bytes = public_key.to_pkcs1_der()?.to_vec();
    Ok(bytes)
}

pub fn rsa_public_key_from_pkcs1_der(
    bytes: &[u8],
) -> Result<RsaPublicKey, RsaSerializationError> {
    let public_key = RsaPublicKey::from_pkcs1_der(bytes)?;
    Ok(public_key)
}

pub fn rsa_private_key_from_pkcs1_pem(
    private_key_pem: &str,
) -> Result<RsaPrivateKey, RsaSerializationError> {
    let private_key = RsaPrivateKey::from_pkcs1_pem(private_key_pem)?;
    Ok(private_key)
}

pub fn rsa_private_key_to_pkcs8_pem(
    private_key: &RsaPrivateKey,
) -> Result<String, RsaSerializationError> {
    let private_key_pem = private_key.to_pkcs8_pem(LineEnding::LF)
        .map(|val| val.to_string())?;
    Ok(private_key_pem)
}

pub fn rsa_private_key_from_pkcs8_pem(
    private_key_pem: &str,
) -> Result<RsaPrivateKey, RsaSerializationError> {
    let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)?;
    Ok(private_key)
}

pub fn rsa_public_key_to_pkcs8_pem(
    public_key: &RsaPublicKey,
) -> Result<String, RsaSerializationError> {
    let public_key_pem = public_key.to_public_key_pem(LineEnding::LF)
        .map_err(rsa::pkcs8::Error::from)?;
    Ok(public_key_pem)
}

pub fn deserialize_rsa_public_key(
    public_key_pem: &str,
) -> Result<RsaPublicKey, RsaSerializationError> {
    if public_key_pem.contains("BEGIN RSA PUBLIC KEY") {
        let public_key = RsaPublicKey::from_pkcs1_pem(public_key_pem.trim())?;
        return Ok(public_key);
    };
    // rsa package can't decode PEM string with non-standard wrap width,
    // so the input should be normalized first
    let parsed_pem = pem::parse(public_key_pem.trim().as_bytes())?;
    let normalized_pem = pem::encode(&parsed_pem);
    let public_key = RsaPublicKey::from_public_key_pem(&normalized_pem)
        .map_err(rsa::pkcs8::Error::from)?;
    Ok(public_key)
}

/// RSASSA-PKCS1-v1_5 signature
pub fn create_rsa_sha256_signature(
    private_key: &RsaPrivateKey,
    message: &str,
) -> Result<Vec<u8>, RsaError> {
    let signing_key = SigningKey::<Sha256>::new_with_prefix(private_key.clone());
    let signature = signing_key.sign(message.as_bytes());
    Ok(signature.to_vec())
}

pub fn verify_rsa_sha256_signature(
    public_key: &RsaPublicKey,
    message: &str,
    signature: &[u8],
) -> bool {
    let verifying_key = VerifyingKey::<Sha256>::new_with_prefix(public_key.clone());
    let signature = match Signature::try_from(signature) {
        Ok(signature) => signature,
        Err(_) => return false,
    };
    let is_valid = verifying_key.verify(
        message.as_bytes(),
        &signature,
    ).is_ok();
    is_valid
}

#[cfg(test)]
mod tests {
    use crate::base64;
    use super::*;

    #[test]
    fn test_private_key_pkcs1_der_encode_decode() {
        let private_key = generate_weak_rsa_key().unwrap();
        let encoded = rsa_private_key_to_pkcs1_der(&private_key).unwrap();
        let decoded = rsa_private_key_from_pkcs1_der(&encoded).unwrap();
        assert_eq!(decoded, private_key);
    }

    #[test]
    fn test_public_key_pkcs1_der_encode_decode() {
        let private_key = generate_weak_rsa_key().unwrap();
        let public_key = RsaPublicKey::from(private_key);
        let encoded = rsa_public_key_to_pkcs1_der(&public_key).unwrap();
        let decoded = rsa_public_key_from_pkcs1_der(&encoded).unwrap();
        assert_eq!(decoded, public_key);
    }

    #[test]
    fn test_private_key_pkcs8_pem_encode_decode() {
        let private_key = generate_weak_rsa_key().unwrap();
        let encoded = rsa_private_key_to_pkcs8_pem(&private_key).unwrap();
        let decoded = rsa_private_key_from_pkcs8_pem(&encoded).unwrap();
        assert_eq!(decoded, private_key);
    }

    #[test]
    fn test_deserialize_rsa_public_key_nowrap() {
        let public_key_pem = "-----BEGIN PUBLIC KEY-----\nMIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQC8ehqQ7n6+pw19U8q2UtxE/9017STW3yRnnqV5nVk8LJ00ba+berqwekxDW+nw77GAu3TJ+hYeeSerUNPup7y3yO3V
YsFtrgWDQ/s8k86sNBU+Ce2GOL7seh46kyAWgJeohh4Rcrr23rftHbvxOcRM8VzYuCeb1DgVhPGtA0xULwIDAQAB\n-----END PUBLIC KEY-----";
        let result = deserialize_rsa_public_key(&public_key_pem);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_deserialize_rsa_public_key_pkcs1() {
        let public_key_pem = "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEA2vzT/2X+LqqoLFLJAZDMGRoAaXEyw9NBCGpu6wTqczs2KEGHdbQe\nIEGKr/+tP6ENOtwe74I2cCsKOPCzUMWTqu2JRd7zfDXUmQnzIZ9wp3AZQ6YFZspj\nxNAzC3dIR6dQr0feebqZZ3n/t7n1ch04Onc2SINyS7MLQHxNi9HTkH9OXZSHDazP\nT8T90Zr2oxo16nVs8rxTVxtE/6bZai90xrSEOfvJfE/0fwb5BK9Fw3J4yv5h+4ck\nrUoSFGEBrTRGgwCrp3UDt/K6Lp4loVC9jzyRMJ5bo5n1rZNgjNCqEqBrJFu6AWSC\nWW/eqkdipgI2IlRezppu0balvEwEluPhNwIDAQAB\n-----END RSA PUBLIC KEY-----\n\n";
        let result = deserialize_rsa_public_key(&public_key_pem);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_public_key_serialization_deserialization() {
        let private_key = generate_weak_rsa_key().unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let public_key_pem = rsa_public_key_to_pkcs8_pem(&public_key).unwrap();
        let public_key = deserialize_rsa_public_key(&public_key_pem).unwrap();
        assert_eq!(public_key, RsaPublicKey::from(&private_key));
    }

    #[test]
    fn test_verify_rsa_signature() {
        let public_key_pem = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1wOPLWAp6nNT5CwttzFP
kWKm+U+weptmldt+EC0JcSuc8sWwki4BU5k/4zunCCc0jxN4ZrfAv3LmYDAx4nbC
Z+7ndKFrDjLtcHMsBBmb+/YYH4lXBXmauMLYGVMZg/8/xPh/euzu+u7wBtFLXU1D
j9PKrqccKZ1I2ENQxTCPMdCI4BYR9niZcKjqG4lVKIbb4VCzIITlVJL7KNt2ZyYX
IjxLKfnZVfCkQ9t5EWkoBME8Gf8hKltxcA5jvEbgHxwmFKgIeSZXg3gQncQL1/qZ
8AVcpaMTTqahxPCFRExlRU0y8ppGcqymyMH/P6jHclRZDqxtwT/S3nFPbwuBAx4O
NwIDAQAB
-----END PUBLIC KEY-----";
        let message = "test";
        let signature = "NFiY1Vx+jZizdiLvS4JAoxcsCI2+SjwWPdWsj8ICqRuMcMg0Gu7/qPu2n/B8sUjXycZH0sUcATIbHaf7AtPTNEU/FDFP+1wR5K4fCEt6QpaV4uGR8KBYTJUV2vE6nnx2Hkr/bAhK8JM3f4OQATqxDc7Ozmosd48sx3alxOOGgZnQD3kCKVhaSJH/ZkYAcPmY7ksSbm9iFX09D2ytEp+FDAD3pzgiNq/MlmozAmSdX9/cS2IFbKAjiJ3wq1T4NqApTZ0Rd8HYuBveMnW3GVeyPalao7uIaYyJumqaf9cBg9l9EkwGwJZ5gsoAV5OHgMTU5bMGF1ShR5xWCnG8fq1ylg==";

        let public_key = deserialize_rsa_public_key(public_key_pem).unwrap();
        let signature_bytes = base64::decode(signature).unwrap();
        let is_valid = verify_rsa_sha256_signature(
            &public_key,
            message,
            &signature_bytes,
        );
        assert_eq!(is_valid, true);
    }

    #[test]
    fn test_create_and_verify_rsa_signature() {
        let private_key = generate_weak_rsa_key().unwrap();
        let message = "test".to_string();
        let signature = create_rsa_sha256_signature(
            &private_key,
            &message,
        ).unwrap();
        let public_key = RsaPublicKey::from(&private_key);

        let is_valid = verify_rsa_sha256_signature(
            &public_key,
            &message,
            &signature,
        );
        assert_eq!(is_valid, true);
    }
}