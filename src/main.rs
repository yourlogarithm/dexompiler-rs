mod opcode;
mod instruction;

use dex::DexReader;
use instruction::Instruction;

fn main() {
    let dex = DexReader::from_file("resources/classes.dex").unwrap();
    let mut counter = 0;
    for class in dex.classes() {
        if let Ok(class) = class {
            for method in class.methods() {
                println!("{:?}.{}", class.jtype().type_descriptor(), method.name());
                if let Some(code) = method.code() {
                    let bytecode_sequence = code.insns().iter().map(|x| x.to_le_bytes()).flatten().collect::<Vec<u8>>();
                    let mut offset = 0;
                    let mut instructions = vec![];
                    while offset < bytecode_sequence.len() {
                        if let Some(inst) = Instruction::try_from_bytes(&bytecode_sequence, offset).unwrap() {
                            println!("{:?}", inst);
                            offset += inst.length() as usize;
                            instructions.push(inst);
                        } else {
                            break;
                        }
                    }
                }
                counter += 1;
            }
        }
    }
}
