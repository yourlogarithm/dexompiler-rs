use axmldecoder::Element;
use serde::Serialize;

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct IntentFilter {
    pub action: String,
}

impl From<Element> for IntentFilter {
    fn from(value: Element) -> Self {
        for child in value.children {
            if let axmldecoder::Node::Element(mut element) = child {
                if element.get_tag() == "action" {
                    if let Some(action) = element.attributes.remove("android:name") {
                        return IntentFilter { action };
                    }
                }
            }
        }
        IntentFilter::default()
    }
}
