use std::fs::File;
use std::io::{read_to_string, Read};
use std::iter::Peekable;

use asn1_rs::{Any, DerSequence, FromDer, Sequence, Set};
use openssl::cms::CMSOptions;
use openssl::md::MdRef;
use std::str::Chars;

use openssl::x509::store::X509StoreRef;
use openssl::x509::X509;
use rustc_hash::FxHashMap;

use crate::errors::VerificationError;
use zip::read::ZipFile;

const DIGEST_KEY_SUFFIX: &'static str = "-Digest";
const DIGEST_MANIFEST_SUFFIX: &'static str = "-Digest-Manifest";

/// https://docs.oracle.com/en/java/javase/17/docs/specs/jar/jar.html#jar-manifest
#[derive(Debug)]
pub struct Manifest {
    file_name: String,
    digest_alg_name: Option<String>,
    main_attribs: FxHashMap<String, String>,
    name_digests: FxHashMap<String, (String, String)>,
}

/// https://datatracker.ietf.org/doc/html/rfc5652
#[derive(Debug, DerSequence)]
pub struct ContentInfo<'a> {
    content_type: Any<'a>,
    #[tag_explicit(0)]
    signed_data: SignedData<'a>,
}

#[derive(Debug, DerSequence)]
pub struct SignedData<'a> {
    version: i32,
    digest_algorithms: Set<'a>,
    encap_content_info: Sequence<'a>,
    #[tag_implicit(0)]
    #[optional]
    certificates: Option<Set<'a>>,
    #[tag_implicit(1)]
    #[optional]
    crls: Option<Set<'a>>,
}

impl Manifest {
    pub fn parse<R>(file_name: &str, r: R) -> Result<Self, anyhow::Error>
    where
        R: Read,
    {
        let data = read_to_string(r)?;
        let mut buf = data.chars().peekable();
        let mut main_attribs = FxHashMap::default();
        let mut name_digests = FxHashMap::default();

        let mut digest_alg_name = None;

        loop {
            let l = Manifest::read_line(&mut buf);
            if let None = l {
                break;
            }

            let kv = Manifest::get_key_val(&l);
            if let None = kv {
                continue;
            }

            let (k, v) = kv.unwrap();
            if k == "Name" {
                loop {
                    // until XXX-Digest key is found
                    let next_line = Manifest::read_line(&mut buf);
                    if let None = next_line {
                        break;
                    }
                    let next_kv = Manifest::get_key_val(&next_line);
                    if let None = next_kv {
                        break; // a newline encountered and there is no XXX-Digest value for this Name
                    }
                    let (key, value) = next_kv.unwrap();
                    if key.ends_with(DIGEST_KEY_SUFFIX) {
                        let alg = key.replace(DIGEST_KEY_SUFFIX, "");
                        let digest = value.trim().to_string();
                        let class_entry = v.trim().to_string();
                        name_digests.insert(class_entry, (alg, digest));
                        break;
                    }
                }
            } else {
                if k.ends_with(DIGEST_MANIFEST_SUFFIX) {
                    digest_alg_name = Some(k.replace(DIGEST_MANIFEST_SUFFIX, ""));
                }
                main_attribs.insert(k.to_string(), v.trim().to_string());
            }
        }

        Ok(Manifest {
            file_name: file_name.to_string(),
            main_attribs,
            name_digests,
            digest_alg_name,
        })
    }

    fn read_line(buf: &mut Peekable<Chars>) -> Option<String> {
        let mut line = String::with_capacity(128);
        let space = &' ';
        loop {
            let char = buf.next();
            if let None = char {
                return None;
            }
            let char = char.unwrap();
            match char {
                '\n' => {
                    let next = buf.peek();
                    if let Some(c) = next {
                        if c != space {
                            break;
                        }

                        if c == space {
                            buf.next(); // consume the space and continue
                        }
                    }
                }
                '\r' => {
                    let next = buf.peek();
                    if let Some(c) = next {
                        if c == &'\n' {
                            continue;
                        }

                        if c != space {
                            break;
                        }

                        if c == space {
                            buf.next(); // consume the space and continue
                        }
                    }
                }
                _ => {
                    line.push(char);
                }
            }
        }
        Some(line)
    }

    fn get_key_val(line: &Option<String>) -> Option<(&str, &str)> {
        if let None = line {
            return None;
        }

        let line = line.as_ref().unwrap();
        if line.is_empty() {
            return None;
        }

        let mut tokens = line.splitn(2, ":");
        let k = tokens.next().or(Some(""));
        let v = tokens.next().or(Some(""));

        Some((k.unwrap(), v.unwrap()))
    }
}

pub fn verify_jar(file_path: &str, cert_store: &X509StoreRef) -> Result<(), VerificationError> {
    let f = File::open(file_path)?;
    let mut za = zip::ZipArchive::new(f)?;

    let mut signatures = Vec::new();
    const META_INF_PREFIX_PATH: &'static str = "META-INF/";
    const DOT_SF_SUFFIX: &'static str = ".SF";
    const MC_SIGNATURE_FILE: &'static str = "META-INF/SERVER.SF";

    {
        // give preference to the MC's signature file, if exists
        if let Ok(_) = za.by_name(MC_SIGNATURE_FILE) {
            signatures.push((MC_SIGNATURE_FILE.to_string(), "SERVER".to_string()));
        }
    }
    if signatures.is_empty() {
        for name in za.file_names() {
            //println!("{}", name);
            if name.starts_with(META_INF_PREFIX_PATH) && name.ends_with(DOT_SF_SUFFIX) {
                let sf_block_prefix = name
                    .replace(META_INF_PREFIX_PATH, "")
                    .replace(DOT_SF_SUFFIX, "");
                signatures.push((name.to_string(), sf_block_prefix));
            }
        }
    }

    //println!("{:?}", signatures);
    let manifest_buf;
    {
        let mut manifest_entry_file = za.by_name("META-INF/MANIFEST.MF")?;
        manifest_buf = read_file(&mut manifest_entry_file)?;
    }

    let manifest = Manifest::parse("MANIFEST.MF", manifest_buf.as_slice())?;
    //println!("{:?}", manifest);

    if signatures.is_empty() {
        return Err(VerificationError {
            cert: None,
            msg: format!("{} is not signed", file_path),
        });
    }

    for (sf_name, sb_prefix) in signatures {
        let mut sigblock: Option<(&str, Vec<u8>)> = None;
        for suffix in ["RSA", "DSA", "EC"] {
            let entry = za.by_name(&format!("META-INF/{}.{}", sb_prefix, suffix));
            if let Ok(mut entry) = entry {
                let entry = read_file(&mut entry)?;
                sigblock = Some((suffix, entry));
                break;
            }
        }

        if let Some((_sig_alg_name, sigblock)) = sigblock {
            let sigmanifest_buf;
            {
                let mut sigmanifest_entry = za.by_name(&sf_name)?;
                sigmanifest_buf = read_file(&mut sigmanifest_entry)?;
            }
            let sigmanifest = Manifest::parse(&sf_name, sigmanifest_buf.as_slice())?;

            let sigblock = sigblock.as_slice();
            let cert = extract_cert(sigblock)?;

            // https://docs.oracle.com/en/java/javase/20/docs/specs/man/jarsigner.html
            // #1 Verify the signature of the .SF file.
            println!("verifying {} of {}", sf_name, file_path);
            let mut cms_info = openssl::cms::CmsContentInfo::from_der(sigblock)?;
            let r = cms_info.verify(
                None,
                Some(cert_store),
                Some(sigmanifest_buf.as_slice()),
                None,
                CMSOptions::empty(),
            );
            if let Err(e) = r {
                let msg = e.to_string();
                println!("verification error: {}", msg);
                if !msg.contains("certificate purpose") { // could be <[unsupported|unsuitable] certificate purpose>
                    // FIXME find a better way to tell OpenSSL to not check the certificate extensions
                    if msg.contains("cms_signerinfo_verify_cert") {
                        return Err(VerificationError { cert, msg });
                    }
                    return Err(VerificationError { cert: None, msg });
                }
            }

            // #2 Verify the digest listed in each entry in the .SF file with each corresponding section in the manifest.
            if let None = sigmanifest.digest_alg_name {
                return Err(VerificationError {
                    cert: None,
                    msg: String::from("missing XXX-Digest-Manifest attribute"),
                });
            }

            let sig_digest_alg_name = sigmanifest.digest_alg_name.unwrap();
            let key = format!("{}{}", sig_digest_alg_name, DIGEST_MANIFEST_SUFFIX);
            let sf_manifest_digest = sigmanifest.main_attribs.get(&key);
            if let None = sf_manifest_digest {
                return Err(VerificationError {
                    cert: None,
                    msg: format!("attribute {} not found in {}", key, sf_name),
                });
            }
            let sf_manifest_digest = sf_manifest_digest.unwrap();

            let digest_ref = get_digest_ref(&sig_digest_alg_name);
            if let None = digest_ref {
                return Err(VerificationError {
                    cert: None,
                    msg: format!("unsupported digest algorithm {}", sig_digest_alg_name),
                });
            }
            let digest_ref = digest_ref.unwrap();

            let mut computed_digest_output: Vec<u8> = vec![0; digest_ref.size()];
            let computed_digest_output = computed_digest_output.as_mut_slice();
            // verify that the digests are same
            let mut ctx = openssl::md_ctx::MdCtx::new().unwrap();
            ctx.digest_init(digest_ref)?;
            ctx.digest_update(manifest_buf.as_slice())?;
            ctx.digest_final(computed_digest_output)?;
            let computed_manifest_digest = openssl::base64::encode_block(computed_digest_output);
            if &computed_manifest_digest != sf_manifest_digest {
                return Err(VerificationError {
                    cert: None,
                    msg: format!("mismatch in manifest digests of {}", file_path),
                });
            }

            // #3 Read each file in the JAR file that has an entry in the .SF file. While reading, compute the file's digest and
            // compare the result with the digest for this file in the manifest section. The digests should be the same or verification fails.
            let mut buf: Vec<u8> = Vec::with_capacity(512);
            for (jar_entry_name, (jar_entry_digest_alg, _jar_entry_digest)) in
                &sigmanifest.name_digests
            {
                let mut ctx = openssl::md_ctx::MdCtx::new().unwrap();
                ctx.digest_init(digest_ref)?;
                let f = za.by_name(jar_entry_name);
                if let Err(ref e) = f {
                    println!(
                        "entry {} not found in {} {}",
                        jar_entry_name,
                        file_path,
                        e.to_string()
                    );
                    continue;
                }
                let mut f = f.unwrap();
                if f.is_dir() {
                    println!(
                        "entry {} of {} is a directory, skipping digest check",
                        jar_entry_name, file_path
                    );
                    continue;
                }
                f.read_to_end(&mut buf)?;
                ctx.digest_update(buf.as_slice())?;
                ctx.digest_final(computed_digest_output)?;

                let computed_digest = openssl::base64::encode_block(&computed_digest_output);
                let (_m_alg, m_digest) = manifest
                    .name_digests
                    .get(jar_entry_name)
                    .expect("missing MANIFEST entry"); // safe to unwrap
                                                       //println!("comparing digests [{} === {}] for {}", m_digest, computed_digest, jar_entry_name);
                if m_digest != &computed_digest {
                    let msg = format!(
                        "{} digest mismatch(manifest={} != computed={}) for {} in {}",
                        jar_entry_digest_alg, m_digest, computed_digest, jar_entry_name, file_path
                    );
                    return Err(VerificationError { cert: None, msg });
                }
                buf.clear();
            }
            println!("verified");
        }
    }
    Ok(())
}

fn get_digest_ref(name: &str) -> Option<&MdRef> {
    use openssl::md::Md;
    match name {
        "SHA-256" => Some(Md::sha256()),
        "SHA-384" => Some(Md::sha384()),
        "SHA-512" => Some(Md::sha512()),
        _ => None,
    }
}
fn extract_cert(sigblock: &[u8]) -> Result<Option<X509>, anyhow::Error> {
    let (_, ci) = ContentInfo::from_der(sigblock).unwrap();
    //println!("{:?}", ci);
    if let Some(cert_set) = ci.signed_data.certificates {
        let cert = X509::from_der(cert_set.content.as_ref())?;
        return Ok(Some(cert));
    }
    Ok(None)
}

fn read_file(zf: &mut ZipFile) -> Result<Vec<u8>, anyhow::Error> {
    let mut buf = Vec::with_capacity(512);
    zf.read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use openssl::ssl::SslFiletype;
    use openssl::x509::store::{HashDir, X509Lookup, X509StoreBuilder};
    use std::collections::VecDeque;
    // use asn1::{ParseResult, SimpleAsn1Writable, WriteBuf, Writer};
    use super::*;

    #[test]
    pub fn test_parse_manifest() {
        let file_name = "MANIFEST.MF";
        let f = File::open("test-resources/MANIFEST.MF").unwrap();
        let m = Manifest::parse(file_name, f).expect("failed to parse the manifest file");
        assert_eq!(file_name, m.file_name);
        assert_eq!(None, m.digest_alg_name);

        let mut main_attribs = FxHashMap::default();
        main_attribs.insert("Created-By", "Apache Maven 3.6.0");
        main_attribs.insert("Application-Name", "Catapult Test Jar");
        main_attribs.insert("Build-Jdk", "1.8.0_352");
        main_attribs.insert("Built-By", "dbugger");
        main_attribs.insert("url", "");
        main_attribs.insert("authors", "Sereen Systems: Kiran Ayyagari");
        main_attribs.insert("Manifest-Version", "1.0");

        for (k, v) in main_attribs {
            assert_eq!(Some(&String::from(v)), m.main_attribs.get(k));
        }

        let mut name_digests = FxHashMap::default();
        name_digests.insert(
            "META-INF/maven/com.sereen.catapult/catapult-test-jar/pom.xml",
            ("SHA-256", "hYrjJTvk33E2hMAm3jQFv94npqhurT1xC/89tZnhrpM="),
        );
        name_digests.insert(
            "log4j.properties",
            ("SHA-256", "qDNFTmmOPAopORClhI9oAJiLlPQLgoBBmz2MTWVTq34="),
        );
        name_digests.insert(
            "META-INF/maven/com.sereen.catapult/catapult-test-jar/pom.properties",
            ("SHA-256", "EuvP5v5Pd2IOFjVJhMixzxIKy2baBE6a+hOWhtAyA/s="),
        );
        name_digests.insert(
            "com/sereen/catapult/App.class",
            ("SHA-256", "YD7chnl2dQvq+IPXfOPOw/82gctW0ZDXrqlVTprcPIs="),
        );

        for (k, (alg, digest)) in name_digests {
            assert_eq!(
                Some(&(String::from(alg), String::from(digest))),
                m.name_digests.get(k)
            );
        }
    }

    #[test]
    pub fn test_parse_signature_file() {
        let file_name = "RSA.SF";
        let f = File::open("test-resources/RSA.SF").unwrap();
        let m = Manifest::parse(file_name, f).expect("failed to parse the signature file");
        assert_eq!(file_name, m.file_name);
        assert_eq!(Some(String::from("SHA-256")), m.digest_alg_name);

        let mut main_attribs = FxHashMap::default();
        main_attribs.insert("Signature-Version", "1.0");
        main_attribs.insert(
            "SHA-256-Digest-Manifest-Main-Attributes",
            "SrvXwDOQW2uH7eiPwlfR+ZwyjWW9AbEfM7dU3f4rDKo=",
        );
        main_attribs.insert(
            "SHA-256-Digest-Manifest",
            "VncmygtfITJAO9mhhNipU9kWkFhAMqFErwtkfZsGXBc=",
        );
        main_attribs.insert("Created-By", "1.8.0_352 (Azul Systems, Inc.)");

        for (k, v) in main_attribs {
            assert_eq!(Some(&String::from(v)), m.main_attribs.get(k));
        }

        let mut name_digests = FxHashMap::default();
        name_digests.insert(
            "META-INF/maven/com.sereen.catapult/catapult-test-jar/pom.xml",
            ("SHA-256", "GUlGP/Ve5YYCc4jxXqE5XHpWLeLJshKzu2k8m9ulumE="),
        );
        name_digests.insert(
            "log4j.properties",
            ("SHA-256", "WZrTZ8yDNvEiIP9ZT1eLvyzRwwvQayYN5m8SY9QKQ4Q="),
        );
        name_digests.insert(
            "META-INF/maven/com.sereen.catapult/catapult-test-jar/pom.properties",
            ("SHA-256", "lEBFiKk6dpR0QEag30N+lOIQKOnGT17wKb8e/YNbWv4="),
        );
        name_digests.insert(
            "com/sereen/catapult/App.class",
            ("SHA-256", "MGAQ6snGyZKVKzAcSfzmq6+4KnwYK3lXBHl25PRKPMU="),
        );

        for (k, (alg, digest)) in name_digests {
            assert_eq!(
                Some(&(String::from(alg), String::from(digest))),
                m.name_digests.get(k)
            );
        }
    }

    #[test]
    pub fn test_parse_content_info() {
        let mut f = File::open("test-resources/RSA.RSA").unwrap();
        let mut buf = Vec::with_capacity(512);
        let r = f.read_to_end(&mut buf).unwrap();
        let (_, ci) = ContentInfo::from_der(buf.as_slice()).unwrap();
        println!("{:?}", ci);
        let cert = X509::from_der(ci.signed_data.certificates.unwrap().content.as_ref());
        println!("{:?}", cert);
    }

    #[test]
    pub fn test_verify() {
        let jar_file = "test-resources/valid-signed.jar";
        let mut xb = X509StoreBuilder::new().unwrap();
        let store = xb.build();
        let r = verify_jar(jar_file, store.as_ref());
        println!("{:?}", r);
        assert!(r.is_err());
        let ve = r.err().unwrap();
        println!("{}", ve.to_json());
        let cert = ve.cert.unwrap();
        let mut xb = X509StoreBuilder::new().unwrap();
        xb.add_cert(cert).unwrap();
        let store = xb.build();
        let r = verify_jar(jar_file, store.as_ref());
        println!("{:?}", r);
        assert!(r.is_ok());
    }

    #[test]
    fn test_verify_failures() {
        let files = [
            "test-resources/tampered-app-class.jar",
            "test-resources/tampered-sf.jar",
        ];
        let mut xb = X509StoreBuilder::new().unwrap();
        let store = xb.build();
        for f in files {
            let r = verify_jar(f, store.as_ref());
            assert!(r.is_err());
        }
    }
}
