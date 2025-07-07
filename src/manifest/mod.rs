mod component;
mod intent_filter;
mod metadata;

use axmldecoder::{Node, ParseError, XmlDocument};
use serde::Serialize;
use std::collections::HashSet;

use crate::manifest::component::Component;

#[derive(Debug, Default, Serialize)]
pub struct Manifest {
    #[serde(rename = "pkg")]
    pub package: Option<String>,

    #[serde(rename = "prm")]
    pub permissions: HashSet<String>,

    #[serde(rename = "act")]
    pub activities: Vec<Component>,

    #[serde(rename = "svc")]
    pub services: Vec<Component>,

    #[serde(rename = "rcv")]
    pub receivers: Vec<Component>,

    #[serde(rename = "prv")]
    pub providers: Vec<Component>,
}

macro_rules! push_component {
    ($element:expr, $manifest:expr, $where:expr) => {{
        let mut component = Component::from($element);
        if let Some(package) = $manifest.package.as_ref() {
            if let Some(name) = component.name.strip_prefix(package.as_str()) {
                component.name = name.into();
            }
        }
        $where.push(component);
    }};
}

pub fn parse(buf: &[u8]) -> Result<Option<Manifest>, ParseError> {
    let XmlDocument { root } = axmldecoder::parse(buf)?;
    match root {
        Some(Node::Element(root)) => {
            let package = root.attributes.get("package").map(|s| s.into());
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
                                    manifest.permissions.insert(perm.into());
                                }
                            }
                        }
                        "application" => {
                            for node in element.children {
                                if let Node::Element(element) = node {
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
            log::warn!("Unexpected root node: {other:?}");
            Ok(None)
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use crate::manifest::{component::Component, intent_filter::IntentFilter, metadata::Metadata};

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
        assert_eq!(manifest.package, Some("com.test.dexompiler".into()));
        assert_eq!(
            manifest.permissions,
            vec!["INTERNET".into(), "FOREGROUND_SERVICE".into()]
                .into_iter()
                .collect()
        );
        assert_eq!(
            manifest.activities,
            vec![Component {
                name: "com.test.TestActivity".into(),
                permission: None,
                intent_filters: vec![],
                metadata: None,
            }]
        );
        assert_eq!(
            manifest.services,
            vec![Component {
                name: "com.test.TestService".into(),
                permission: None,
                intent_filters: vec![],
                metadata: None,
            }]
        );
        assert_eq!(
            manifest.receivers,
            vec![Component {
                name: "com.test.TestReceiver".into(),
                permission: None,
                intent_filters: vec![],
                metadata: None,
            }]
        );
        assert_eq!(
            manifest.providers,
            vec![Component {
                name: "com.test.TestProvider".into(),
                permission: None,
                intent_filters: vec![],
                metadata: Some(Metadata {
                    name: "android.support.FILE_PROVIDER_PATHS".into()
                }),
            }]
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
        assert_eq!(manifest.package, Some("com.hzmjdbvq.segiyntr".into()));
        assert_eq!(
            manifest.permissions,
            vec![
                "ACCESS_NETWORK_STATE".into(),
                "SYSTEM_ALERT_WINDOW".into(),
                "INTERNET".into()
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            manifest.activities,
            [
                "com.win.first.MainActivity",
                "com.win.first.LogoActivity",
                "com.win.first.WebActivity"
            ]
            .into_iter()
            .map(|s| Component {
                name: (*s).into(),
                permission: None,
                intent_filters: if s == "com.win.first.MainActivity" {
                    vec![IntentFilter {
                        action: "android.intent.action.MAIN".into(),
                    }]
                } else {
                    vec![]
                },
                metadata: None,
            })
            .collect::<Vec<_>>()
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
        assert_eq!(manifest.package, Some("com.asnad.nightparty".into()));
        assert_eq!(
            manifest.permissions,
            vec![
                "INTERNET".into(),
                "RECEIVE_SMS".into(),
                "READ_PHONE_STATE".into(),
                "READ_SMS".into(),
                "FOREGROUND_SERVICE".into(),
                "POST_NOTIFICATIONS".into(),
                "WRITE_EXTERNAL_STORAGE".into(),
                "ACCESS_NETWORK_STATE".into()
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            manifest.activities,
            [
                ".Last2",
                ".LastPage",
                ".SplashActivity",
                ".MainActivity",
                "com.google.android.gms.common.api.GoogleApiActivity"
            ]
            .into_iter()
            .map(|s| Component {
                name: s.into(),
                permission: None,
                intent_filters: if s == ".SplashActivity" {
                    vec![IntentFilter {
                        action: "android.intent.action.MAIN".into(),
                    }]
                } else {
                    vec![]
                },
                metadata: None,
            })
            .collect::<Vec<_>>()
        );
        assert_eq!(
            manifest.services,
            [
                ".SmsSerivce",
                "com.google.firebase.components.ComponentDiscoveryService"
            ]
            .into_iter()
            .map(|s| Component {
                name: s.into(),
                permission: None,
                intent_filters: vec![],
                metadata: if s == "com.google.firebase.components.ComponentDiscoveryService" {
                    Some(Metadata {
                        name: "com.google.firebase.components:com.google.firebase.database.DatabaseRegistrar"
                            .into(),
                    })
                } else {
                    None
                },
            })
            .collect::<Vec<_>>()
        );
        assert_eq!(
            manifest.receivers,
            vec![
                Component {
                    name: ".SmsReceiver".into(),
                    permission: Some("android.permission.BROADCAST_SMS".into()),
                    intent_filters: vec![IntentFilter {
                        action: "android.provider.Telephony.SMS_RECEIVED".into(),
                    }],
                    metadata: None,
                },
                Component {
                    name: "androidx.profileinstaller.ProfileInstallReceiver".into(),
                    permission: Some("android.permission.DUMP".into()),
                    intent_filters: vec![
                        IntentFilter {
                            action: "androidx.profileinstaller.action.INSTALL_PROFILE".into()
                        },
                        IntentFilter {
                            action: "androidx.profileinstaller.action.SKIP_FILE".into()
                        },
                        IntentFilter {
                            action: "androidx.profileinstaller.action.SAVE_PROFILE".into()
                        },
                        IntentFilter {
                            action: "androidx.profileinstaller.action.BENCHMARK_OPERATION".into()
                        }
                    ],
                    metadata: None,
                },
            ]
        );
        assert_eq!(
            manifest.providers,
            [
                "com.google.firebase.provider.FirebaseInitProvider",
                "androidx.startup.InitializationProvider"
            ]
            .into_iter()
            .map(|s| Component {
                name: (*s).into(),
                permission: None,
                intent_filters: vec![],
                metadata: if s == "androidx.startup.InitializationProvider" {
                    Some(Metadata {
                        name: "androidx.profileinstaller.ProfileInstallerInitializer".into(),
                    })
                } else {
                    None
                },
            })
            .collect::<Vec<_>>()
        );
    }
}
