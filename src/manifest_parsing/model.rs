use crate::utils::Error;
use axmldecoder::{Node, XmlDocument};
use pyo3::pyclass;


#[derive(Debug, Default, Clone)]
#[pyclass]
pub struct ManifestParseModel {
    /// The permissions requested by the app
    pub permissions: Vec<String>
}

impl ManifestParseModel {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let xml = axmldecoder::parse(bytes)?;
        let XmlDocument { root } = xml;
        let permissions: Option<Vec<String>> = match root { 
            Some(Node::Element(root)) => {
                Some(root.children.into_iter()
                .filter_map(|node| match node {
                    Node::Element(mut element) if element.get_tag() == "uses-permission" => {
                        element.attributes.remove("android:name")
                    },
                    _ => None
                })
                .filter_map(|s| match s.strip_prefix("android.permission.") {
                    Some(s) => Some(s.to_string()),
                    None => None
                })
                .collect())
            },
            _ => None,
        };
        Ok(Self { permissions: permissions.unwrap_or_default() })
    }
}