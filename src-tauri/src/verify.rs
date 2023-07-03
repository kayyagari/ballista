use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use anyhow::Error;
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct Manifest {
    file_name: String,
    main_attribs: FxHashMap<String, String>,
    name_digests: FxHashMap<String, (String, String)>,
}

const DIGEST_KEY_SUFFIX: &'static str = "-Digest";

impl Manifest {
    pub fn parse<R>(file_name: &str, r: R) -> Result<Self, Error>
    where R : Read
    {
        let mut lines = BufReader::new(r).lines();
        let mut name_val = String::from("");
        let mut prev_key = String::from("");

        let mut main_attribs = FxHashMap::default();
        let mut name_digests = FxHashMap::default();

        loop {
            let l = lines.next();
            if let None = l {
                break;
            }
            if let Some(l) = l {
                if let Ok(l) = l {
                    let mut tokens = l.splitn(2, ":");
                    let k = tokens.next();
                    if let None = k {
                        continue;
                    }
                    if let Some(k) = k {
                        if k.starts_with(" ") && prev_key == "Name" {
                            let k = k.trim();
                            name_val.push_str(k);
                            continue;
                        }

                        prev_key = k.to_string();
                        let v = tokens.next();
                        if k == "Name" {
                            if let Some(v) = v {
                                name_val = v.trim().to_string();
                            }
                        }
                        else if k.ends_with(DIGEST_KEY_SUFFIX) {
                            let alg = k.replace(DIGEST_KEY_SUFFIX, "");
                            if let Some(v) = v {
                                let digest = v.trim().to_string();
                                name_digests.insert(name_val.clone(), (alg, digest));
                                name_val.truncate(0);
                            }
                        }
                        else  {
                            if let Some(v) = v {
                                main_attribs.insert(k.to_string(), v.trim().to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(Manifest{file_name: file_name.to_string(), main_attribs, name_digests})
    }
}

// pub fn verify_jar(file_path: &str) -> Result<(), Error> {
//     let f = File::open(file_path)?;
//     let mut za = zip::ZipArchive::new(f)?;
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_parse_manifest() {
        let file_name = "MANIFEST.MF";
        let f = File::open("test-resources/MANIFEST.MF").unwrap();
        let m = Manifest::parse(file_name, f).expect("failed to parse the manifest file");
        assert_eq!(file_name, m.file_name);

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
        name_digests.insert("META-INF/maven/com.sereen.catapult/catapult-test-jar/pom.xml", ("SHA-256", "hYrjJTvk33E2hMAm3jQFv94npqhurT1xC/89tZnhrpM="));
        name_digests.insert("log4j.properties", ("SHA-256", "qDNFTmmOPAopORClhI9oAJiLlPQLgoBBmz2MTWVTq34="));
        name_digests.insert("META-INF/maven/com.sereen.catapult/catapult-test-jar/pom.properties", ("SHA-256", "EuvP5v5Pd2IOFjVJhMixzxIKy2baBE6a+hOWhtAyA/s="));
        name_digests.insert("com/sereen/catapult/App.class", ("SHA-256", "YD7chnl2dQvq+IPXfOPOw/82gctW0ZDXrqlVTprcPIs="));

        for (k, (alg, digest)) in name_digests {
            assert_eq!(Some(&(String::from(alg), String::from(digest))), m.name_digests.get(k));
        }
    }
}