use chrono::Utc;
use http::Method;
use sha2::{Digest, Sha256};

use crate::{
    base64,
    crypto_rsa::{
        create_rsa_sha256_signature,
        RsaError,
        RsaPrivateKey,
    },
};

const HTTP_SIGNATURE_ALGORITHM: &str = "rsa-sha256";
const HTTP_SIGNATURE_DATE_FORMAT: &str = "%a, %d %b %Y %T GMT";

pub struct HttpSignatureHeaders {
    pub host: String,
    pub date: String,
    pub digest: Option<String>,
    pub signature: String,
}

#[derive(thiserror::Error, Debug)]
pub enum HttpSignatureError {
    #[error("invalid request url")]
    UrlError(#[from] url::ParseError),

    #[error("signing error")]
    SigningError(#[from] RsaError),
}

fn get_message_digest(message: &str) -> String {
    let digest = Sha256::digest(message.as_bytes());
    let digest_b64 = base64::encode(digest);
    digest_b64
}

/// Creates HTTP signature according to the old HTTP Signatures Spec:
/// https://datatracker.ietf.org/doc/html/draft-cavage-http-signatures.
pub fn create_http_signature(
    request_method: Method,
    request_url: &str,
    request_body: &str,
    signer_key: &RsaPrivateKey,
    signer_key_id: &str,
) -> Result<HttpSignatureHeaders, HttpSignatureError> {
    let request_url_object = url::Url::parse(request_url)?;
    let request_target = format!(
        "{} {}",
        request_method.as_str().to_lowercase(),
        request_url_object.path(),
    );
    // TODO: Host header may contain port
    let host = request_url_object.host_str()
        .ok_or(url::ParseError::EmptyHost)?
        .to_string();
    let date = Utc::now().format(HTTP_SIGNATURE_DATE_FORMAT).to_string();
    let maybe_digest = if request_body.is_empty() {
        None
    } else {
        let digest = format!(
            "SHA-256={}",
            get_message_digest(request_body),
        );
        Some(digest)
    };

    let mut headers = vec![
        ("(request-target)", &request_target),
        ("host", &host),
        ("date", &date),
    ];
    if let Some(ref digest) = maybe_digest {
        headers.push(("digest", digest));
    };

    let message = headers.iter()
        .map(|(name, value)| format!("{}: {}", name, value))
        .collect::<Vec<String>>()
        .join("\n");
    let headers_parameter = headers.iter()
        .map(|(name, _)| name.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    let signature = create_rsa_sha256_signature(signer_key, &message)?;
    let signature_parameter = base64::encode(signature);
    let signature_header = format!(
        r#"keyId="{}",algorithm="{}",headers="{}",signature="{}""#,
        signer_key_id,
        HTTP_SIGNATURE_ALGORITHM,
        headers_parameter,
        signature_parameter,
    );
    let headers = HttpSignatureHeaders {
        host,
        date,
        digest: maybe_digest,
        signature: signature_header,
    };
    Ok(headers)
}
