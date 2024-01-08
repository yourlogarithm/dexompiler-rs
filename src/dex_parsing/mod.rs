use std::{hash::Hash, collections::HashMap};

use dex::Dex;
mod instruction;
mod opcode;

use self::instruction::Instruction;

pub(crate) fn parse_dexes(dexes: Vec<Dex<impl AsRef<[u8]>>>, sequence_cap: usize) -> (Vec<u8>, Vec<(usize, usize)>) {
    let mut op_seq = vec![]; 
    let mut method_bounds = vec![];
    let mut pos = 0;
    for dex in dexes {
        let (curr_op_seq, curr_method_bounds) = get_op_seq(dex, &mut pos);
        op_seq.extend(curr_op_seq);
        method_bounds.extend(curr_method_bounds);
    }
    if sequence_cap > 0 {
        method_bounds.retain(|(start, end)| *start < sequence_cap && *end < sequence_cap);
        if let Some(last) = method_bounds.last() {
            let cap = last.1;
            op_seq.truncate(cap + 1);
        }
    }
    (op_seq, method_bounds)
}


fn get_op_seq(dex: Dex<impl AsRef<[u8]>>, pos: &mut usize) -> (Vec<u8>, Vec<(usize, usize)>) {
    let mut op_seq = vec![];
    let mut m_bounds = vec![];
    let mut edges = HashMap::new();
    let mut i = 0;
    for class in dex.classes().filter_map(Result::ok) {
        for code in class.methods().filter_map(|m| m.code().map(|c| c.insns())) {
            let mut offset = 0;
            let mut out_edges = vec![];
            let mut current_method_seq = vec![];
            let mut do_extend = true;
            let start = *pos;
            while offset < code.len() {
                match Instruction::try_from_code(code, offset) {
                    Ok(Some((inst, length))) => {
                        offset += length;
                        current_method_seq.push(*inst.opcode() as u8);
                        if let Some(m_idx) = inst.m_idx() {
                            out_edges.push(*m_idx);
                        }
                    },
                    Ok(None) => break,
                    Err(_) => {
                        // eprintln!("Error parsing: {}::{}", class.jtype().to_java_type(), method.name());
                        do_extend = false;
                        break;
                    },
                }
            }
            if do_extend && !current_method_seq.is_empty() {
                *pos += current_method_seq.len();
                m_bounds.push((start, *pos - 1));
                op_seq.extend(current_method_seq);
                // TODO: Fix this
                edges.insert(i, out_edges); 
                i += 1;
            }
        }
    }
    (op_seq, m_bounds)
}
