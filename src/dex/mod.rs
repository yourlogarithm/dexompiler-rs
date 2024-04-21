mod errors;
mod instruction;
mod opcode;

use std::collections::HashMap;

pub use self::{errors::DexError, opcode::Opcode};
use crate::dex::instruction::Instruction;
use dex::Dex;
use itertools::Itertools;
use log::{debug, error};
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq, Hash)]
pub struct Method {
    pub fullname: String,
    pub insns: Vec<Instruction>,
}

pub fn get_methods(
    dexes: &[Dex<impl AsRef<[u8]>>],
    regexes: Option<Vec<Regex>>,
) -> Result<Vec<Method>, DexError> {
    // Extract methods
    let mut call_graph = HashMap::new();
    let mut name_map = HashMap::new();
    for dex in dexes.into_iter() {
        for class in dex.classes().filter_map(Result::ok) {
            for method in class.methods() {
                if let Some(code) = method.code() {
                    let mut offset = 0;
                    let bytecode = code.insns();
                    let mut insns = Vec::new();
                    let fullname = class.jtype().to_string()
                        + method.name()
                        + &method
                            .params()
                            .iter()
                            .map(|p| p.to_string())
                            .collect::<String>()
                        + &method.return_type().to_string();
                    let mut calls = Vec::new();
                    while let Some((inst, len)) = Instruction::try_from_code(bytecode, offset)
                        .map_err(|source| DexError {
                            class_name: class.jtype().to_java_type(),
                            method_name: method.name().to_string(),
                            source,
                        })?
                    {
                        if let Some(m_idx) = inst.m_idx {
                            match dex.get_method_item(m_idx as u64) {
                                Ok(method_item) => {
                                    match (
                                        dex.get_type(method_item.class_idx() as u32),
                                        dex.get_string(method_item.name_idx() as u32),
                                    ) {
                                        (Ok(t), Ok(n)) => {
                                            calls.push(t.to_java_type() + "." + &n);
                                        }
                                        (Err(e), _) | (_, Err(e)) => error!("{e}"),
                                    }
                                }
                                Err(e) => error!("{e}"),
                            }
                        }
                        insns.push(inst);
                        offset += len;
                    }
                    call_graph.insert(fullname.clone(), calls);
                    name_map.insert(fullname.clone(), Method { fullname, insns });
                }
            }
        }
    }

    // Sort so the manifest components will be prioritized
    let mut flattened = Vec::with_capacity(call_graph.len());
    let mut stack: Vec<_> = if let Some(regexes) = regexes {
        debug!("Sorting by manifest components");
        call_graph
            .keys()
            .sorted_by_key(|m| !regexes.iter().any(|r| r.is_match(m)))
            .collect()
    } else {
        debug!("Sorting by method name");
        call_graph.keys().sorted().collect()
    };

    // DFS to flatten the call graph
    while let Some(method_name) = stack.pop() {
        if let Some(method) = name_map.remove(method_name) {
            flattened.push(method);
            if let Some(adjacent_methods) = call_graph.get(method_name) {
                stack.extend(adjacent_methods.iter().rev());
            }
        }
    }

    Ok(flattened)
}
