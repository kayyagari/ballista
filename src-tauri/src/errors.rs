use openssl::error::ErrorStack;
use openssl::x509::{X509NameRef, X509};
use rustc_hash::FxHashMap;
use serde_json::{Number, Value};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::io::Error;
use openssl::hash::MessageDigest;
use zip::result::ZipError;

#[derive(Debug)]
pub struct VerificationError {
    pub(crate) cert: Option<X509>,
    pub(crate) msg: String,
}

impl VerificationError {
    pub fn to_json(&self) -> String {
        let mut obj = FxHashMap::default();
        obj.insert("msg", Value::String(self.msg.clone()));
        obj.insert("code", Value::Number(Number::from(1)));
        if let Some(ref cert) = self.cert {
            let mut cert_details = serde_json::Map::new();
            let der = match cert.to_der() {
                Ok(d) => d,
                Err(_) => return format!("{{\"msg\":\"{}\",\"code\":1}}", self.msg),
            };
            let der = openssl::base64::encode_block(der.as_slice());
            cert_details.insert("der".to_string(), Value::String(der));
            let subject = format_name(cert.subject_name());
            cert_details.insert("subject".to_string(), Value::String(subject));

            let issuer = format_name(cert.issuer_name());
            cert_details.insert("issuer".to_string(), Value::String(issuer));

            let expires_on = cert.not_after().to_string();
            cert_details.insert("expires_on".to_string(), Value::String(expires_on));

            let sha256_sum_bytes = match cert.digest(MessageDigest::sha256()) {
                Ok(d) => d,
                Err(_) => return format!("{{\"msg\":\"{}\",\"code\":1}}", self.msg),
            };
            let sha256_string = hex::encode(sha256_sum_bytes);
            cert_details.insert("sha256sum".to_string(), Value::String(sha256_string));

            obj.insert("cert", Value::Object(cert_details));
        }

        serde_json::to_string(&obj).unwrap_or_else(|_| format!("{{\"msg\":\"{}\",\"code\":1}}", self.msg))
    }
}

impl Display for VerificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<std::io::Error> for VerificationError {
    fn from(value: Error) -> Self {
        VerificationError {
            cert: None,
            msg: value.to_string(),
        }
    }
}

impl From<ZipError> for VerificationError {
    fn from(value: ZipError) -> Self {
        VerificationError {
            cert: None,
            msg: value.to_string(),
        }
    }
}

impl From<anyhow::Error> for VerificationError {
    fn from(value: anyhow::Error) -> Self {
        VerificationError {
            cert: None,
            msg: value.to_string(),
        }
    }
}

impl From<ErrorStack> for VerificationError {
    fn from(value: ErrorStack) -> Self {
        VerificationError {
            cert: None,
            msg: value.to_string(),
        }
    }
}

fn format_name(name: &X509NameRef) -> String {
    let mut parts = VecDeque::new();
    let mut formatted_name = String::with_capacity(128);
    for e in name.entries() {
        let nid = e.object().nid().short_name().unwrap_or("??");
        let val = e.data().as_utf8()
            .map(|s| s.to_string())
            .unwrap_or_else(|_| String::from("??"));
        parts.push_front(format!("{}={}", nid, val));
    }

    if let Some(last_part) = parts.pop_back() {
        for p in &parts {
            formatted_name.push_str(p);
            formatted_name.push(',');
        }
        formatted_name.push_str(&last_part);
    }

    formatted_name
}
