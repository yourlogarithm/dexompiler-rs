mod errors;
mod instruction;
mod opcode;

use std::collections::HashMap;

pub use self::{errors::DexError, opcode::Opcode};
use crate::dex::instruction::Instruction;
use bitcode::{Decode, Encode};
use dex::Dex;
use log::{debug, error};
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize, Encode, Decode, PartialEq, Eq, Hash)]
pub struct Method {
    /// String in format `class_type` + `method_name` + `params.join()` + `return_type`
    #[serde(rename = "fn")]
    pub fullname: String,
    #[serde(rename = "ins")]
    pub insns: Vec<Instruction>,
}

macro_rules! get_fullname {
    ($class_type:expr, $method_name:expr, $params:expr, $return_type:expr) => {
        $class_type.to_string()
            + $method_name
            + $params
                .iter()
                .map(|p| p.to_string())
                .collect::<String>()
                .as_str()
            + $return_type.to_string().as_str()
    };
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
                    let fullname = get_fullname!(
                        class.jtype(),
                        method.name(),
                        method.params(),
                        method.return_type()
                    );
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
                                        dex.get_proto_item(method_item.proto_idx() as u64),
                                    ) {
                                        (Ok(t), Ok(n), Ok(p)) => {
                                            match dex.get_type(p.return_type()) {
                                                Ok(r) => {
                                                    if p.params_off() == 0 {
                                                        calls.push(
                                                            t.to_string()
                                                                + &n.to_string()
                                                                + &r.to_string(),
                                                        )
                                                    } else {
                                                        match dex.get_interfaces(p.params_off()) {
                                                            Ok(params) => calls.push(
                                                                get_fullname!(t, &n, params, r),
                                                            ),
                                                            Err(e) => error!("{e}"),
                                                        }
                                                    }
                                                }
                                                Err(e) => error!("{e}"),
                                            }
                                        }
                                        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                                            error!("{e}")
                                        }
                                    }
                                }
                                Err(e) => error!("{e}"),
                            }
                        }
                        insns.push(inst);
                        offset += len;
                    }
                    let method = Method { fullname, insns };
                    call_graph.insert(method.fullname.clone(), calls);
                    name_map.insert(method.fullname.clone(), method);
                }
            }
        }
    }

    // Sort so the manifest components will be prioritized
    let mut flattened = Vec::with_capacity(call_graph.len());
    let mut stack: Vec<_> = call_graph.keys().collect();
    if let Some(regexes) = regexes {
        debug!("Sorting by manifest components");
        stack.sort_by_cached_key(|&name| {
            (
                regexes.iter().any(|r| r.is_match(name)),
                std::cmp::Reverse(name),
            )
        });
    } else {
        debug!("Sorting by method name");
        stack.sort_by_key(|n| std::cmp::Reverse(*n));
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

#[cfg(test)]
mod tests {
    use crate::dex::{instruction::Instruction, Opcode};
    use dex::DexReader;

    use super::get_methods;

    #[test]
    fn test_hello_world() {
        let dex = DexReader::from_file("tests/dex/hello_world.dex").unwrap();
        let methods = get_methods(&[dex], None).unwrap();

        let init = &methods[0];
        assert_eq!(init.fullname, "LTestBasic;<init>V");
        assert_eq!(
            init.insns,
            vec![
                Instruction {
                    opcode: Opcode::InvokeDirect,
                    m_idx: Some(3)
                },
                Instruction {
                    opcode: Opcode::ReturnVoid,
                    m_idx: None
                }
            ]
        );

        let main = &methods[1];
        assert_eq!(main.fullname, "LTestBasic;main[Ljava/lang/String;V");
        assert_eq!(
            main.insns,
            vec![
                Instruction {
                    opcode: Opcode::SgetObject,
                    m_idx: None
                },
                Instruction {
                    opcode: Opcode::ConstString,
                    m_idx: None
                },
                Instruction {
                    opcode: Opcode::InvokeVirtual,
                    m_idx: Some(2)
                },
                Instruction {
                    opcode: Opcode::ReturnVoid,
                    m_idx: None
                }
            ]
        );
    }

    #[test]
    fn test_call_graph() {
        let dex = DexReader::from_file("tests/dex/call_graph.dex").unwrap();
        let methods = get_methods(&[dex], None).unwrap();
        
        assert_eq!(methods[0].fullname, "LCallGraph;<init>V");
        assert_eq!(methods[1].fullname, "LCallGraph;aV");
        assert_eq!(methods[2].fullname, "LCallGraph;zV");
        assert_eq!(methods[3].fullname, "LCallGraph;yV");
        assert_eq!(methods[4].fullname, "LCallGraph;xV");
        assert_eq!(methods[5].fullname, "LCallGraph;main[Ljava/lang/String;V");
    }
}
