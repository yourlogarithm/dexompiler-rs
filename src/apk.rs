use crate::manifest::Manifest;
use crate::dex::Method;

use bitcode::{Decode, Encode};
use serde::Serialize;

/// Represents an APK (Android Package) with metadata and methods.
#[derive(Debug, Serialize, Encode, Decode)]
pub struct Apk {
    #[serde(rename = "man")]
    pub manifest: Option<Manifest>,
    /// Topologically DFS sorted methods in the DEX(es) where:
    /// * Class name is present in AndroidManifest.xml (if available) is major order
    /// * Method signature is minor order
    /// 
    /// This means that the methods will be sorted using the order above first, 
    /// then a DFS traversal will be done to flatten the call graph.
    #[serde(rename = "mth")]
    pub methods: Vec<Method>,
}

impl Apk {
    /// Converts the APK to a compact representation with reduced method information.
    ///
    /// ### Returns
    /// A `CompactApk` with the same manifest and compacted method data.
    pub fn to_compact(self) -> CompactApk {
        self.into()
    }
}

/// A compact representation of a method with opcode data.
///
/// This is useful for reducing the APK size while maintaining method information.
pub type CompactMethod = Vec<u8>;

/// A compact version of the `Apk` struct where methods are stored as a vector of opcodes.
///
/// This reduces the APK's overall size and can be used for efficient storage or transfer.
#[derive(Debug, Serialize, Encode, Decode)]
pub struct CompactApk {
    /// Optional manifest information
    #[serde(rename = "man")]
    pub manifest: Option<Manifest>,
    /// Compact method representations as opcode vectors.
    #[serde(rename = "mth")]
    pub methods: Vec<CompactMethod>,
}

/// Converts an `Apk` into a `CompactApk` by compacting method information.
///
/// This transformation reduces the overall size of the APK.
impl From<Apk> for CompactApk {
    fn from(apk: Apk) -> Self {
        CompactApk {
            manifest: apk.manifest,
            methods: apk
                .methods
                .into_iter()
                .map(|method| method.insns.into_iter().map(|i| i.opcode as u8).collect())
                .collect(),
        }
    }
}
