mod opcodes;

use dex::DexReader;

fn main() {
    let dex = DexReader::from_file("resources/classes.dex").unwrap();
    for class in dex.classes() {
        if let Ok(class) = class {
            for method in class.methods() {
                if let Some(code) = method.code() {
                    code.insns()
                }
            }
        }
    }
}
