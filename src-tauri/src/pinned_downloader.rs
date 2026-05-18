// Licensed under the MPL-2.0 License. See LICENSE file in the project root.

use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::{error::Error, fmt::{Display, Formatter}};

use reqwest::blocking::{Client, Response};
use reqwest::tls::TlsInfo;
use reqwest::Url;

#[derive(Debug)]
pub enum PinnedDownloadError {
    PeerCertificate { peer_cert_der: Vec<u8> },
    Other(String),
}

impl Display for PinnedDownloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PinnedDownloadError::PeerCertificate { .. } => write!(f, "peer certificate mismatch"),
            PinnedDownloadError::Other(message) => write!(f, "{}", message),
        }
    }
}

impl Error for PinnedDownloadError {}

pub fn download_to_string(url: &str, expected_cert: Option<&[u8]>) -> Result<String, PinnedDownloadError> {
    checked_get(url, expected_cert)?
        .text()
        .map_err(|error| PinnedDownloadError::Other(format!("failed to read response body: {error}")))
}

pub fn download_to_file(url: &str, expected_cert: Option<&[u8]>, destination: &Path) -> Result<(), PinnedDownloadError> {
    let mut response = checked_get(url, expected_cert)?;
    let mut file = File::create(destination)
        .map_err(|error| PinnedDownloadError::Other(format!("failed to create {}: {error}", destination.display())))?;
    copy(&mut response, &mut file)
        .map(|_| ())
        .map_err(|error| PinnedDownloadError::Other(format!("failed to write {}: {error}", destination.display())))
}

fn checked_get(url: &str, expected_cert: Option<&[u8]>) -> Result<Response, PinnedDownloadError> {
    let url = parse_https_url(url)?;

    let client = Client::builder()
        .https_only(true)
        .tls_info(true)
        // We will do our own cert validation, so we need to disable the client's built in checks
        .danger_accept_invalid_certs(true)
        // Since we're relying on pinning, we can be a bit more lax about hostname verification
        .danger_accept_invalid_hostnames(true)
        .build()
        .map_err(|error| PinnedDownloadError::Other(format!("failed to build HTTP client: {error}")))?;

    let response = client
        .get(url)
        .send()
        .map_err(|error| PinnedDownloadError::Other(format!("request failed: {error}")))?;

    let peer_cert_der = response
        .extensions()
        .get::<TlsInfo>()
        .and_then(TlsInfo::peer_certificate)
        .map(|der| der.to_vec())
        .ok_or_else(|| PinnedDownloadError::Other("peer certificate not available".to_string()))?;

    // If we don't have an expected cert, or if it doesn't match, return err
    if expected_cert != Some(peer_cert_der.as_slice()) {
        return Err(PinnedDownloadError::PeerCertificate { peer_cert_der });
    }

    if !response.status().is_success() {
        return Err(PinnedDownloadError::Other(format!(
            "request failed with HTTP status {}",
            response.status()
        )));
    }

    Ok(response)
}

fn parse_https_url(raw_url: &str) -> Result<Url, PinnedDownloadError> {
    let url = Url::parse(raw_url)
        .map_err(|error| PinnedDownloadError::Other(format!("invalid URL: {error}")))?;

    if url.scheme() != "https" {
        return Err(PinnedDownloadError::Other(format!("URL must use https: {raw_url}")));
    }

    Ok(url)
}
