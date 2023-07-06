use std::fmt::{Display, Formatter};
use std::io::Error;
use openssl::error::ErrorStack;
use openssl::x509::X509;
use rustc_hash::FxHashMap;
use zip::result::ZipError;
use serde_json::{Number, Value};

#[derive(Debug)]
pub struct VerificationError {
    pub(crate) cert: Option<X509>,
    pub(crate) msg: String
}

impl VerificationError {
    pub fn to_json(&self) -> Result<String, anyhow::Error> {
        let mut obj = FxHashMap::default();
        obj.insert("msg", Value::String(self.msg.clone()));
        obj.insert("code", Value::Number(Number::from(1)));
        if let Some(ref cert) = self.cert {
            let mut cert_details = serde_json::Map::new();
            let der = cert.to_der()?;
            let der = openssl::base64::encode_block(der.as_slice());
            cert_details.insert("der".to_string(), Value::String(der));
            let subject = format!("{:?}", cert.subject_name());
            cert_details.insert("subject".to_string(), Value::String(subject));

            obj.insert("cert", Value::Object(cert_details));
        }

        let json = serde_json::to_string(&obj)?;
        Ok(json)
    }
}
impl Display for VerificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<std::io::Error> for VerificationError {
    fn from(value: Error) -> Self {
        VerificationError{cert: None, msg: value.to_string()}
    }
}

impl From<ZipError> for VerificationError {
    fn from(value: ZipError) -> Self {
        VerificationError{cert: None, msg: value.to_string()}
    }
}

impl From<anyhow::Error> for VerificationError {
    fn from(value: anyhow::Error) -> Self {
        VerificationError{cert: None, msg: value.to_string()}
    }
}

impl From<ErrorStack> for VerificationError {
    fn from(value: ErrorStack) -> Self {
        VerificationError{cert: None, msg: value.to_string()}
    }
}