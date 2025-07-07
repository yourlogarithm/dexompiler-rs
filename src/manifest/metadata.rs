use axmldecoder::Element;
use serde::Serialize;

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct Metadata {
    pub name: String,
}

impl From<Element> for Metadata {
    fn from(mut value: Element) -> Self {
        Metadata {
            name: value.attributes.remove("android:name").unwrap_or_default(),
        }
    }
}
