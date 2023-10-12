use crate::opcode::Opcode;

#[derive(Debug)]
enum Type {
    String,
    Type,
    InstanceField,
    StaticField,
    Method,
    MethodHandle,
    MethodPrototype,
    CallSite,
    Prototype
}

#[derive(Debug)]
enum Format {
    ImmediateSignedByte(i8),
    ConstantPoolIndex16(Type, u16),
    ConstantPoolIndex32(Type, u32),
    ImmediateSignedHat(i16),
    ImmediateSigned32(u32),  // i32 or f32
    ImmediateSigned64(u64),    // i64 or f64
    ImmediateSignedNibble(i8),
    ImmediateSignedShort(i16),
    BranchTarget8(i8),
    BranchTarget16(i16),
    BranchTarget32(i32),
}

#[derive(Debug)]
pub struct Instruction {
    /// The opcode of the instruction
    pub opcode: Opcode,
    /// Non-register arguments to the instruction
    arguments: Vec<Format>,
    /// The length of the instruction
    length: u8 ,
    /// The offset of the instruction
    offset: usize,
}


macro_rules! extract_from_le {
    ($type:ty, $arr:expr, $idx:expr) => {{
        let size = std::mem::size_of::<$type>();
        let mut byte_slice: Vec<u8> = vec![];
        for i in 0..size {
            byte_slice.push($arr[$idx + i]);
        }
        <$type>::from_le_bytes(byte_slice.try_into().unwrap())
    }};
}


impl Instruction {
    pub fn length(&self) -> u8 {
        self.length
    }

    pub fn try_from_bytes(bytecode: &[u8], offset: usize) -> Result<Option<Self>, &'static str> {
        let bytes = &bytecode[offset..];
        if let Ok(opcode) = Opcode::try_from(bytes[0]) {
            if let Ok::<(Vec<Format>, u8), &'static str>((arguments, length)) = match bytes[0] {
                0x0 => {
                    if bytes.len() > 1 && (1..=3).contains(&bytes[1]) {
                        return Ok(None);
                    } else {
                        Ok((vec![], 2))
                    }
                },
                0x1 | 0x4 | 0x7 | 0xa..=0x11 | 0x1d | 0x1e | 0x21 | 0x27 | 0x7b..=0x8f | 0xb0..=0xcf => Ok((vec![], 2)),
                0x2 | 0x5 | 0x8 | 0x2d..=0x31 | 0x44..=0x51 | 0x90..=0xaf => Ok((vec![], 4)),
                0x3 | 0x6 | 0x9 => Ok((vec![], 6)),
                0xd8..=0xe2 => Ok((vec![Format::ImmediateSignedByte(bytes[3] as i8)], 4)),
                0x1a => Ok((vec![Format::ConstantPoolIndex16(Type::String, extract_from_le!(u16, bytes, 2))], 4)),
                0x1c | 0x1f | 0x20 | 0x22 | 0x23 => Ok((vec![Format::ConstantPoolIndex16(Type::Type, extract_from_le!(u16, bytes, 2))], 4)),
                0x52..=0x5f => Ok((vec![Format::ConstantPoolIndex16(Type::InstanceField, extract_from_le!(u16, bytes, 2))], 4)),
                0x60..=0x6d => Ok((vec![Format::ConstantPoolIndex16(Type::StaticField, extract_from_le!(u16, bytes, 2))], 4)),
                0xfe => Ok((vec![Format::ConstantPoolIndex16(Type::MethodHandle, extract_from_le!(u16, bytes, 2))], 4)),
                0xff => Ok((vec![Format::ConstantPoolIndex16(Type::MethodPrototype, extract_from_le!(u16, bytes, 2))], 4)),
                0x1b => Ok((vec![Format::ConstantPoolIndex32(Type::String, extract_from_le!(u32, bytes, 2))], 6)),
                0x24 | 0x25 => Ok((vec![Format::ConstantPoolIndex32(Type::Type, extract_from_le!(u32, bytes, 2))], 6)),
                0x6e..=0x72 | 0x74..=0x78 => Ok((vec![Format::ConstantPoolIndex32(Type::Method, extract_from_le!(u32, bytes, 2))], 6)),
                0xfc | 0xfd => Ok((vec![Format::ConstantPoolIndex32(Type::CallSite, extract_from_le!(u32, bytes, 2))], 6)),
                0xfa | 0xfb => Ok((vec![Format::ConstantPoolIndex16(Type::Method, extract_from_le!(u16, bytes, 2)), Format::ConstantPoolIndex16(Type::Prototype, 6)], 8)),
                0x15 | 0x19 => Ok((vec![Format::ImmediateSignedHat(extract_from_le!(i16, bytes, 2))], 4)),
                0x14 | 0x17 => Ok((vec![Format::ImmediateSigned32(extract_from_le!(u32, bytes, 2))], 6)),
                0x18 => Ok((vec![Format::ImmediateSigned64(extract_from_le!(u64, bytes, 2))], 10)),
                0x12 => Ok((vec![Format::ImmediateSignedNibble(bytes[1] as i8)], 2)),
                0x13 | 0x16 | 0xd0..=0xd7 => Ok((vec![Format::ImmediateSignedShort(extract_from_le!(i16, bytes, 2))], 4)),
                0x28 => Ok((vec![Format::BranchTarget8(bytes[1] as i8)], 2)),
                0x29 | 0x32..=0x3d => Ok((vec![Format::BranchTarget16(extract_from_le!(i16, bytes, 2))], 4)),
                0x26 | 0x2a..=0x2c => Ok((vec![Format::BranchTarget32(extract_from_le!(i32, bytes, 2))], 6)),
                0x3e..=0x43 | 0x73 | 0x79..=0x7a | 0xe3..=0xf9 => Err("Unimplemented"),
            } {
                return Ok(Some(Instruction { opcode, arguments, length, offset }));
            } else {
                return Err("Invalid instruction")
            }
        }
        return Err("Invalid opcode")
    }
}