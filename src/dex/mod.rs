mod errors;
mod instruction;
mod method;
mod opcode;

use std::collections::HashMap;

use self::method::Signature;
use dex::Dex;
use log::{debug, error};
use regex::Regex;

pub use self::{errors::DexError, instruction::Instruction, method::{Method, CompactMethod}, opcode::Opcode};

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
                    let params = method.params();
                    let signature = if params.is_empty() {
                        Signature::new(class.jtype(), method.name(), None, method.return_type())
                    } else {
                        Signature::new(
                            class.jtype(),
                            method.name(),
                            Some(params),
                            method.return_type(),
                        )
                    };
                    let mut calls = Vec::new();
                    while let Some((inst, len)) = Instruction::try_from_code(bytecode, offset)
                        .map_err(|source| DexError {
                            class_name: class.jtype().to_java_type(),
                            method_name: method.name().to_string(),
                            source,
                        })?
                    {
                        if let Some(m_idx) = inst.method_id {
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
                                                        calls
                                                            .push(Signature::new(&t, &n, None, &r));
                                                    } else {
                                                        match dex.get_interfaces(p.params_off()) {
                                                            Ok(params) => {
                                                                calls.push(Signature::new(
                                                                    &t,
                                                                    &n,
                                                                    Some(&params),
                                                                    &r,
                                                                ))
                                                            }
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
                    let method = Method { signature, insns };
                    call_graph.insert(method.signature.clone(), calls);
                    name_map.insert(method.signature.clone(), method);
                }
            }
        }
    }

    // Sort so the manifest components will be prioritized
    let mut flattened = Vec::with_capacity(call_graph.len());
    let mut stack: Vec<_> = call_graph.keys().collect();
    if let Some(regexes) = regexes {
        debug!("Sorting by manifest components");
        stack.sort_by_cached_key(|&sig| {
            (
                regexes.iter().any(|r| r.is_match(&sig.class_type)),
                std::cmp::Reverse(sig),
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
    use crate::dex::{instruction::Instruction, method::Signature, Opcode};
    use dex::DexReader;

    use super::get_methods;

    #[test]
    fn test_hello_world() {
        let dex = DexReader::from_file("tests/dex/hello_world.dex").unwrap();
        let methods = get_methods(&[dex], None).unwrap();

        let init = &methods[0];
        assert_eq!(
            init.signature,
            Signature {
                class_type: "LTestBasic;".to_string(),
                method_name: "<init>".to_string(),
                params: None,
                return_type: "V".to_string()
            }
        );
        assert_eq!(
            init.insns,
            vec![
                Instruction {
                    opcode: Opcode::InvokeDirect,
                    method_id: Some(3)
                },
                Instruction {
                    opcode: Opcode::ReturnVoid,
                    method_id: None
                }
            ]
        );

        let main = &methods[1];
        assert_eq!(
            main.signature,
            Signature {
                class_type: "LTestBasic;".to_string(),
                method_name: "main".to_string(),
                params: Some(vec!["[Ljava/lang/String;".to_string()]),
                return_type: "V".to_string()
            }
        );
        assert_eq!(
            main.insns,
            vec![
                Instruction {
                    opcode: Opcode::SgetObject,
                    method_id: None
                },
                Instruction {
                    opcode: Opcode::ConstString,
                    method_id: None
                },
                Instruction {
                    opcode: Opcode::InvokeVirtual,
                    method_id: Some(2)
                },
                Instruction {
                    opcode: Opcode::ReturnVoid,
                    method_id: None
                }
            ]
        );
    }

    #[test]
    fn test_call_graph() {
        let dex = DexReader::from_file("tests/dex/call_graph.dex").unwrap();
        let methods = get_methods(&[dex], None).unwrap();
        assert_eq!(
            methods[0].signature,
            Signature {
                class_type: "LCallGraph;".to_string(),
                method_name: "<init>".to_string(),
                params: None,
                return_type: "V".to_string()
            }
        );
        assert_eq!(
            methods[1].signature,
            Signature {
                class_type: "LCallGraph;".to_string(),
                method_name: "a".to_string(),
                params: None,
                return_type: "V".to_string()
            }
        );
        assert_eq!(
            methods[2].signature,
            Signature {
                class_type: "LCallGraph;".to_string(),
                method_name: "z".to_string(),
                params: None,
                return_type: "V".to_string()
            }
        );
        assert_eq!(
            methods[3].signature,
            Signature {
                class_type: "LCallGraph;".to_string(),
                method_name: "y".to_string(),
                params: None,
                return_type: "V".to_string()
            }
        );
        assert_eq!(
            methods[4].signature,
            Signature {
                class_type: "LCallGraph;".to_string(),
                method_name: "x".to_string(),
                params: None,
                return_type: "V".to_string()
            }
        );
        assert_eq!(
            methods[5].signature,
            Signature {
                class_type: "LCallGraph;".to_string(),
                method_name: "main".to_string(),
                params: Some(vec!["[Ljava/lang/String;".to_string()]),
                return_type: "V".to_string()
            }
        );
    }
}
