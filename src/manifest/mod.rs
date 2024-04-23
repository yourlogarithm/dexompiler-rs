use axmldecoder::{Node, ParseError, XmlDocument};
use bitcode::{Decode, Encode};
use log::warn;
use serde::Serialize;

#[derive(Debug, Default, Serialize, Encode, Decode)]
pub struct Manifest {
    pub package: Option<String>,
    pub permissions: Vec<String>,
    pub activities: Vec<String>,
    pub services: Vec<String>,
    pub receivers: Vec<String>,
    pub providers: Vec<String>,
}

macro_rules! push_component {
    ($element:expr, $manifest:expr, $where:expr) => {
        if let Some(name) = $element.attributes.remove("android:name") {
            if let Some(package) = $manifest.package.as_ref() {
                if let Some(name) = name.strip_prefix(package.as_str()) {
                    $where.push(name.to_string());
                } else {
                    $where.push(name);
                }
            }
        }
    };
}

pub fn parse(buf: &[u8]) -> Result<Option<Manifest>, ParseError> {
    let XmlDocument { root } = axmldecoder::parse(buf)?;
    match root {
        Some(Node::Element(root)) => {
            let package = root.attributes.get("package").map(|s| s.to_string());
            let mut manifest = Manifest { package, ..Default::default() };
            for node in root.children {
                match node {
                    Node::Element(mut element) => {
                        let tag = element.get_tag();
                        if tag == "uses-permission" {
                            if let Some(name) = element.attributes.remove("android:name") {
                                if let Some(perm) = name.strip_prefix("android.permission.") {
                                    manifest.permissions.push(perm.to_string());
                                }
                            }
                        } else if tag == "application" {
                            for node in element.children {
                                match node {
                                    Node::Element(mut element) => {
                                        let tag = element.get_tag();
                                        match tag {
                                            "activity" => push_component!(element, &manifest, manifest.activities),
                                            "service" => push_component!(element, &manifest, manifest.services),
                                            "receiver" => push_component!(element, &manifest, manifest.receivers),
                                            "provider" => push_component!(element, &manifest, manifest.providers),
                                            _ => ()
                                        }
                                    },
                                    _ => ()
                                }
                            }
                        }
                    },
                    _ => ()
                }
            }
            Ok(Some(manifest))
        },
        Some(other) => {
            warn!("Unexpected root node: {other:?}");
            Ok(None)
        },
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};
    use super::parse;

    #[test]
    fn test_parse() {
        let mut buf = Vec::new();
        File::open("tests/manifest/AndroidManifest.xml").unwrap().read_to_end(&mut buf).unwrap();
        let manifest = parse(&buf).unwrap().unwrap();
        assert_eq!(manifest.package, Some("com.test.dexompiler".to_string()));
        assert_eq!(manifest.permissions, vec!["INTERNET".to_string(), "FOREGROUND_SERVICE".to_string()]);
        assert_eq!(manifest.activities, vec!["com.test.TestActivity".to_string()]);
        assert_eq!(manifest.services, vec!["com.test.TestService".to_string()]);
        assert_eq!(manifest.receivers, vec!["com.test.TestReceiver".to_string()]);
        assert_eq!(manifest.providers, vec!["com.test.TestProvider".to_string()]);
    }
}