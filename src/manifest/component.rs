use axmldecoder::{Element, Node};
use serde::Serialize;

use crate::manifest::{intent_filter::IntentFilter, metadata::Metadata};

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct Component {
    pub name: String,
    pub intent_filters: Vec<IntentFilter>,
    pub metadata: Option<Metadata>,
    pub permission: Option<String>,
}

impl From<Element> for Component {
    fn from(mut value: Element) -> Self {
        let mut component = Component::default();
        component.name = value.attributes.remove("android:name").unwrap_or_default();

        for child in value.children {
            if let Node::Element(element) = child {
                match element.get_tag() {
                    "intent-filter" => component.intent_filters.push(element.into()),
                    "meta-data" => component.metadata = Some(element.into()),
                    _ => {}
                }
            }
        }

        component.permission = value.attributes.remove("android:permission");
        component
    }
}
