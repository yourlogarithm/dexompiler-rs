use bitcode::{Decode, Encode};
use dex::{jtype::Type, string::DexString};
use serde::Serialize;

use super::instruction::Instruction;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Encode, Decode)]
pub struct Signature {
    #[serde(rename = "ct")]
    pub class_type: String,
    #[serde(rename = "mn")]
    pub method_name: String,
    #[serde(rename = "args", skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<String>>,
    #[serde(rename = "rt")]
    pub return_type: String,
}

impl Signature {
    pub fn new(
        class_type: &Type,
        method_name: &DexString,
        params: Option<&[Type]>,
        return_type: &Type,
    ) -> Self {
        Self {
            class_type: class_type.to_string(),
            method_name: method_name.to_string(),
            params: params.map(|params| params.iter().map(|t| t.to_string()).collect()),
            return_type: return_type.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Encode, Decode, PartialEq, Eq, Hash)]
pub struct Method {
    #[serde(flatten)]
    pub signature: Signature,
    #[serde(rename = "ins")]
    pub insns: Vec<Instruction>,
}
