use axmldecoder::{Node, ParseError, XmlDocument};
use log::warn;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Default, Serialize)]
pub struct Manifest {
    #[serde(rename = "pkg")]
    pub package: Option<String>,
    #[serde(rename = "prm")]
    pub permissions: HashSet<String>,
    #[serde(rename = "act")]
    pub activities: Vec<String>,
    #[serde(rename = "svc")]
    pub services: Vec<String>,
    #[serde(rename = "rcv")]
    pub receivers: Vec<String>,
    #[serde(rename = "prv")]
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
            let mut manifest = Manifest {
                package,
                ..Default::default()
            };
            for node in root.children {
                if let Node::Element(mut element) = node {
                    match element.get_tag() {
                        "uses-permission" => {
                            if let Some(name) = element.attributes.remove("android:name") {
                                if let Some(perm) = name.strip_prefix("android.permission.") {
                                    manifest.permissions.insert(perm.to_string());
                                }
                            }
                        }
                        "application" => {
                            for node in element.children {
                                if let Node::Element(mut element) = node {
                                    match element.get_tag() {
                                        "activity" => {
                                            push_component!(element, &manifest, manifest.activities)
                                        }
                                        "service" => {
                                            push_component!(element, &manifest, manifest.services)
                                        }
                                        "receiver" => {
                                            push_component!(element, &manifest, manifest.receivers)
                                        }
                                        "provider" => {
                                            push_component!(element, &manifest, manifest.providers)
                                        }
                                        _ => (),
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
            Ok(Some(manifest))
        }
        Some(other) => {
            warn!("Unexpected root node: {other:?}");
            Ok(None)
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use std::{fs::File, io::Read};

    #[test]
    fn test_parse_a() {
        let mut buf = Vec::new();
        File::open("tests/manifest/a.xml")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let manifest = parse(&buf).unwrap().unwrap();
        assert_eq!(manifest.package, Some("com.test.dexompiler".to_string()));
        assert_eq!(
            manifest.permissions,
            vec!["INTERNET".to_string(), "FOREGROUND_SERVICE".to_string()]
                .into_iter()
                .collect()
        );
        assert_eq!(
            manifest.activities,
            vec!["com.test.TestActivity".to_string()]
        );
        assert_eq!(manifest.services, vec!["com.test.TestService".to_string()]);
        assert_eq!(
            manifest.receivers,
            vec!["com.test.TestReceiver".to_string()]
        );
        assert_eq!(
            manifest.providers,
            vec!["com.test.TestProvider".to_string()]
        );
    }

    #[test]
    fn test_parse_b() {
        let mut buf = Vec::new();
        File::open("tests/manifest/b.xml")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let manifest = parse(&buf).unwrap().unwrap();
        assert_eq!(manifest.package, Some("com.hzmjdbvq.segiyntr".to_string()));
        assert_eq!(
            manifest.permissions,
            vec![
                "ACCESS_NETWORK_STATE".to_string(),
                "SYSTEM_ALERT_WINDOW".to_string(),
                "INTERNET".to_string()
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            manifest.activities,
            vec![
                "com.win.first.MainActivity".to_string(),
                "com.win.first.LogoActivity".to_string(),
                "com.win.first.WebActivity".to_string()
            ]
        );
        assert_eq!(manifest.services.len(), 0);
        assert_eq!(manifest.receivers.len(), 0);
        assert_eq!(manifest.providers.len(), 0);
    }

    #[test]
    fn test_parse_c() {
        let mut buf = Vec::new();
        File::open("tests/manifest/c.xml")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let manifest = parse(&buf).unwrap().unwrap();
        assert_eq!(manifest.package, Some("com.asnad.nightparty".to_string()));
        assert_eq!(
            manifest.permissions,
            vec![
                "INTERNET".to_string(),
                "RECEIVE_SMS".to_string(),
                "READ_PHONE_STATE".to_string(),
                "READ_SMS".to_string(),
                "FOREGROUND_SERVICE".to_string(),
                "POST_NOTIFICATIONS".to_string(),
                "WRITE_EXTERNAL_STORAGE".to_string(),
                "ACCESS_NETWORK_STATE".to_string()
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            manifest.activities,
            vec![
                ".Last2".to_string(),
                ".LastPage".to_string(),
                ".SplashActivity".to_string(),
                ".MainActivity".to_string(),
                "com.google.android.gms.common.api.GoogleApiActivity".to_string(),
            ]
        );
        assert_eq!(
            manifest.services,
            vec![
                ".SmsSerivce".to_string(),
                "com.google.firebase.components.ComponentDiscoveryService".to_string()
            ]
        );
        assert_eq!(
            manifest.receivers,
            vec![
                ".SmsReceiver".to_string(),
                "androidx.profileinstaller.ProfileInstallReceiver".to_string()
            ]
        );
        assert_eq!(
            manifest.providers,
            vec![
                "com.google.firebase.provider.FirebaseInitProvider".to_string(),
                "androidx.startup.InitializationProvider".to_string()
            ]
        );
    }
}
