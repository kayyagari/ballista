// Copyright (c) Kiran Ayyagari. All rights reserved.
// Copyright (c) Diridium Technologies Inc. All rights reserved.
// Licensed under the MPL-2.0 License. See LICENSE file in the project root.

use openssl::hash::MessageDigest;
use openssl::x509::{X509NameRef, X509};
use rustc_hash::FxHashMap;
use serde_json::{Number, Value};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct PeerDetails {
    pub der: String,
    pub subject: String,
    pub issuer: String,
    pub expires_on: String,
    pub sha256sum: String,
}

impl PeerDetails {
    pub fn from_der(peer_cert_der: &[u8]) -> Result<Self, anyhow::Error> {
        let cert = X509::from_der(peer_cert_der)?;
        let sha256sum = hex::encode(cert.digest(MessageDigest::sha256())?);

        Ok(PeerDetails {
            der: openssl::base64::encode_block(peer_cert_der),
            subject: format_name(cert.subject_name()),
            issuer: format_name(cert.issuer_name()),
            expires_on: cert.not_after().to_string(),
            sha256sum,
        })
    }

    fn to_value(&self) -> Value {
        let mut peer = serde_json::Map::new();
        peer.insert("der".to_string(), Value::String(self.der.clone()));
        peer.insert("subject".to_string(), Value::String(self.subject.clone()));
        peer.insert("issuer".to_string(), Value::String(self.issuer.clone()));
        peer.insert("expires_on".to_string(), Value::String(self.expires_on.clone()));
        peer.insert("sha256sum".to_string(), Value::String(self.sha256sum.clone()));
        Value::Object(peer)
    }
}

#[derive(Debug)]
pub enum LaunchError {
    UntrustedPeer { peer: PeerDetails },
    FingerprintMismatch { peer: PeerDetails, expected_fingerprint: String },
}

impl LaunchError {
    pub fn to_json(&self) -> String {
        let mut obj = FxHashMap::default();

        match self {
            LaunchError::UntrustedPeer { peer } => {
                obj.insert("code", Value::Number(Number::from(1)));
                obj.insert("msg", Value::String("Untrusted peer certificate.".to_string()));
                obj.insert("peer", peer.to_value());
            }
            LaunchError::FingerprintMismatch {
                peer,
                expected_fingerprint,
            } => {
                obj.insert("code", Value::Number(Number::from(2)));
                obj.insert(
                    "msg",
                    Value::String("Server fingerprint does not match the pinned value.".to_string()),
                );
                obj.insert("peer", peer.to_value());
                obj.insert(
                    "expected_fingerprint",
                    Value::String(expected_fingerprint.clone()),
                );
            }
        }

        serde_json::to_string(&obj).unwrap_or_else(|_| "{\"code\":-1,\"msg\":\"launch failed\"}".to_string())
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
