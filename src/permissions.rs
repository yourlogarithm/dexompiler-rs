use std::collections::HashSet;

use axmldecoder::{Node, ParseError, XmlDocument};

pub type Permission = String;

pub fn parse(buf: &[u8]) -> Result<Option<HashSet<Permission>>, ParseError> {
    let XmlDocument { root } = axmldecoder::parse(buf)?;
    let perms = match root {
        Some(Node::Element(root)) => Some(
            root.children
                .into_iter()
                .filter_map(|node| match node {
                    Node::Element(mut element) if element.get_tag() == "uses-permission" => {
                        element.attributes.remove("android:name")
                    }
                    _ => None,
                })
                .filter_map(|s| match s.strip_prefix("android.permission.") {
                    Some(s) => Some(s.to_string()),
                    None => None,
                })
                .collect()
        ),
        _ => None,
    };
    Ok(perms)
}