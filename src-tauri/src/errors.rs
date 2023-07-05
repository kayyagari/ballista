use std::fmt::{Display, Formatter};
use std::io::Error;
use openssl::error::ErrorStack;
use openssl::x509::X509;
use zip::result::ZipError;

#[derive(Debug)]
pub struct VerificationError {
    pub(crate) cert: Option<X509>,
    pub(crate) msg: String
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