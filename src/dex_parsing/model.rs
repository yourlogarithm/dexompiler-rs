use std::collections::{HashMap, VecDeque};
use dex::Dex;
use pyo3::pyclass;
use crate::utils::Error;

use super::instruction::Instruction;

type MethodBound = (usize, usize);

#[pyclass]
#[derive(Debug)]
pub struct DexParseModel {
    /// Flat sequence of opcodes from all methods in all dexes (in topological order & up to sequence_cap)
    pub op_seq: Vec<u8>,
    /// Vector of (start_index, end_index) pairs for each method in the op_seq, the vector is sorted in topological order
    pub method_bounds: Vec<MethodBound>
}

impl DexParseModel {
    pub fn op_seq(&self) -> &[u8] {
        &self.op_seq
    }

    pub fn method_bounds(&self) -> &[(usize, usize)] {
        &self.method_bounds
    }

    pub fn try_from_dexes(dexes: Vec<Dex<impl AsRef<[u8]>>>, sequence_cap: usize) -> Result<Self, Error> {
        let mut op_seq = vec![]; 
        let mut call_graph = HashMap::new();
        let mut pos = 0;
        for dex in dexes {
            let (curr_op_seq, curr_method_bounds) = DexParseModel::get_op_seq(dex, &mut pos, sequence_cap)?;
            op_seq.extend(curr_op_seq);
            call_graph.extend(curr_method_bounds);
            if op_seq.len() >= sequence_cap {
                break;
            }
        }
        
        let sorted_methods = DexParseModel::topological_sort(call_graph);
        let op_seq = DexParseModel::sort_opcode_sequence(op_seq, &sorted_methods);
        
        Ok(Self {
            op_seq,
            method_bounds: sorted_methods,
        })
    }

    fn get_op_seq(
        dex: Dex<impl AsRef<[u8]>>, 
        pos: &mut usize,
        sequence_cap: usize
    ) -> Result<(Vec<u8>, HashMap<MethodBound, Vec<MethodBound>>), Error> {
        let mut local_pos = 0;
        let mut op_seq = vec![];
        let mut edges = HashMap::new();
        let mut id_to_bounds = HashMap::new();
        'outer: for class in dex.classes().filter_map(Result::ok) {
            for (method, code) in class.methods().filter_map(|m| m.code().map(|c| (m, c.insns()))) {
                let mut offset = 0;
                let mut out_edges = vec![];
                let mut current_method_seq = vec![];
                let mut do_extend = true;
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
                            do_extend = false;
                            // eprintln!("Error parsing: {}::{}", class.jtype().to_java_type(), method.name());
                            break;
                        },
                    }
                }
                if do_extend && !current_method_seq.is_empty() {
                    if sequence_cap > 0 && *pos + current_method_seq.len() > sequence_cap {
                        break 'outer;
                    }
                    let bounds = (*pos + local_pos, *pos + local_pos + current_method_seq.len() - 1);
                    id_to_bounds.insert(method.id(), bounds);
                    local_pos += current_method_seq.len();
                    *pos += current_method_seq.len();
                    op_seq.extend(current_method_seq);
                    edges.insert(bounds, out_edges);
                }
            }
        }
        
        let normalized_edges = edges.into_iter().map(|(key, values)| {
            let normalized_values = values.into_iter()
                .filter_map(|id| id_to_bounds.get(&(id as u64)))
                .collect();
            (key, normalized_values)
        }).collect();
            
        
        Ok((op_seq, normalized_edges))
    }

    fn topological_sort(graph: HashMap<MethodBound, Vec<MethodBound>>) -> Vec<MethodBound> {
        let mut in_degree = HashMap::new();
        for (method, calls) in &graph {
            in_degree.entry(*method).or_insert(0);
            for &call in calls {
                *in_degree.entry(call).or_insert(0) += 1;
            }
        }
    
        let mut queue = VecDeque::new();
        for (method, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(*method);
            }
        }
    
        let mut result = Vec::new();
        while let Some(method) = queue.pop_front() {
            result.push(method);
            if let Some(calls) = graph.get(&method) {
                for &call in calls {
                    if let Some(degree) = in_degree.get_mut(&call) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(call);
                        }
                    }
                }
            }
        }
    
        if result.len() == in_degree.len() {
            result
        } else {
            Vec::new() // Cycle detected, returning empty vector
        }
    }

    fn sort_opcode_sequence(opcode_sequence: Vec<u8>, sorted_methods: &[MethodBound]) -> Vec<u8> {
        let mut sorted_sequence = Vec::new();
        for (start, end) in sorted_methods {
            if *start > opcode_sequence.len() || *end > opcode_sequence.len() {
                continue;
            }
            sorted_sequence.extend_from_slice(&opcode_sequence[*start..=*end]);
        }
        sorted_sequence
    }
}