use std::iter::Peekable;

use crate::instruction::{BinopKind, CmpKind, IfTest, InvokeKind, Op, OpType, UnopKind};

use super::{Instruction, InstructionError};

fn word_to_bytes(word: u16) -> (u8, u8) {
    let u2 = word.to_le_bytes();
    (u2[0], u2[1])
}

fn word_to_nibbles(word: u16) -> (u8, u8, u8, u8) {
    let u2 = word.to_le_bytes();
    (u2[0] >> 4, u2[0] & 0x0F, u2[1] >> 4, u2[1] & 0x0F)
}

fn byte_to_nibbles(byte: u8) -> (u8, u8) {
    (byte >> 4, byte & 0x0F)
}

impl Instruction {
    pub fn try_from_code<I: Iterator<Item = u16>>(
        bytecode_iterator: &mut Peekable<I>,
    ) -> Result<Option<Self>, InstructionError> {
        let Some(start) = bytecode_iterator.next() else {
            return Ok(None);
        };
        let (opcode_byte, immediate_args) = word_to_bytes(start);
        macro_rules! next {
            () => {
                bytecode_iterator
                    .next()
                    .ok_or(InstructionError::TooShort(opcode_byte))?
            };
        }
        macro_rules! dword {
            () => {
                (next!() as u32) << 16 | next!() as u32
            };
        }
        macro_rules! qword {
            () => {
                (next!() as u64) << 48
                    | (next!() as u64) << 32
                    | (next!() as u64) << 16
                    | next!() as u64
            };
        }
        let instruction = match opcode_byte {
            0x00 => {
                if (1..=3).contains(&immediate_args) {
                    return Ok(None);
                }
                Self::Nop
            },
            0x01 => {
                let (dst, src) = byte_to_nibbles(immediate_args);
                Self::Move { dst, src }
            }
            0x02 => Self::MoveFrom16 {
                dst: immediate_args,
                src: next!(),
            },
            0x03 => Self::Move16 {
                dst: next!(),
                src: next!(),
            },
            0x04 => {
                let (dst, src) = byte_to_nibbles(immediate_args);
                Self::MoveWide { dst, src }
            }
            0x05 => Self::MoveWideFrom16 {
                dst: immediate_args,
                src: next!(),
            },
            0x06 => Self::MoveWide16 {
                dst: next!(),
                src: next!(),
            },
            0x07 => {
                let (dst, src) = byte_to_nibbles(immediate_args);
                Self::MoveObject { dst, src }
            }
            0x08 => Self::MoveObjectFrom16 {
                dst: immediate_args,
                src: next!(),
            },
            0x09 => Self::MoveObject16 {
                dst: next!(),
                src: next!(),
            },
            0x0A => Self::MoveResult {
                dst: immediate_args,
            },
            0x0B => Self::MoveResultWide {
                dst: immediate_args,
            },
            0x0C => Self::MoveResultObject {
                dst: immediate_args,
            },
            0x0D => Self::MoveException {
                dst: immediate_args,
            },
            0x0E => Self::ReturnVoid,
            0x0F => Self::Return {
                src: immediate_args,
            },
            0x10 => Self::ReturnWide {
                src: immediate_args,
            },
            0x11 => Self::ReturnObject {
                src: immediate_args,
            },
            0x12 => {
                let (dst, value) = byte_to_nibbles(immediate_args);
                Self::Const4 {
                    dst,
                    value: value as i8,
                }
            }
            0x13 => Self::Const16 {
                dst: immediate_args,
                value: next!() as i16,
            },
            0x14 => Self::Const {
                dst: immediate_args,
                value: dword!(),
            },
            0x15 => Self::ConstHigh16 {
                dst: immediate_args,
                value: next!() as i16,
            },
            0x16 => Self::ConstWide16 {
                dst: immediate_args,
                value: next!() as i16,
            },
            0x17 => Self::ConstWide32 {
                dst: immediate_args,
                value: dword!() as i32,
            },
            0x18 => Self::ConstWide {
                dst: immediate_args,
                value: qword!(),
            },
            0x19 => Self::ConstWideHigh16 {
                dst: immediate_args,
                value: next!() as i16,
            },
            0x1A => Self::ConstString {
                dst: immediate_args,
                idx: next!(),
            },
            0x1B => Self::ConstStringJumbo {
                dst: immediate_args,
                idx: dword!(),
            },
            0x1C => Self::ConstClass {
                dst: immediate_args,
                idx: next!(),
            },
            0x1D => Self::MonitorEnter {
                obj: immediate_args,
            },
            0x1E => Self::MonitorExit {
                obj: immediate_args,
            },
            0x1F => Self::CheckCast {
                obj: immediate_args,
                idx: next!(),
            },
            0x20 => {
                let (dst, obj) = byte_to_nibbles(immediate_args);
                Self::InstanceOf {
                    dst,
                    obj,
                    idx: next!(),
                }
            }
            0x21 => {
                let (dst, obj) = byte_to_nibbles(immediate_args);
                Self::ArrayLength { dst, obj }
            }
            0x22 => Self::NewInstance {
                dst: immediate_args,
                idx: next!(),
            },
            0x23 => {
                let (dst, size) = byte_to_nibbles(immediate_args);
                Self::NewArray {
                    dst,
                    size,
                    idx: next!(),
                }
            }
            0x24 => {
                let (size, c) = byte_to_nibbles(immediate_args);
                let (d, e, f, g) = word_to_nibbles(next!());
                let idx = next!();
                Self::FilledNewArray {
                    size,
                    idx,
                    c,
                    d,
                    e,
                    f,
                    g,
                }
            }
            0x25 => Self::FilledNewArrayRange {
                size: immediate_args,
                idx: next!(),
                first: next!(),
            },
            0x26 => Self::FillArrayData {
                arr: immediate_args,
                off: dword!(),
            },
            0x27 => Self::Throw {
                off: immediate_args,
            },
            0x28 => Self::Goto {
                off: immediate_args,
            },
            0x29 => Self::Goto16 {
                off: next!() as i16,
            },
            0x2A => Self::Goto32 {
                off: dword!() as i32,
            },
            0x2B => Self::PackedSwitch {
                reg: immediate_args,
                off: dword!() as i32,
            },
            0x2C => Self::SparseSwitch {
                reg: immediate_args,
                off: dword!() as i32,
            },
            0x2D..=0x31 => {
                let kind = match opcode_byte {
                    0x2D => CmpKind::LtFloat,
                    0x2E => CmpKind::GtFloat,
                    0x2F => CmpKind::LtDouble,
                    0x30 => CmpKind::GtDouble,
                    0x31 => CmpKind::Long,
                    _ => unreachable!(),
                };
                let (src0, src1) = word_to_bytes(next!());
                Self::CmpKind { kind, dst: immediate_args, src0, src1 }
            },
            0x32..=0x37 => {
                let kind = match opcode_byte {
                    0x32 => IfTest::Eq,
                    0x33 => IfTest::Ne,
                    0x34 => IfTest::Lt,
                    0x35 => IfTest::Ge,
                    0x36 => IfTest::Gt,
                    0x37 => IfTest::Le,
                    _ => unreachable!(),
                };
                let (a, b) = byte_to_nibbles(immediate_args);
                Self::IfTest { kind, a, b, off: next!() as i16 }
            },
            0x38..=0x3D => {
                let kind = match opcode_byte {
                    0x38 => IfTest::Eq,
                    0x39 => IfTest::Ne,
                    0x3A => IfTest::Lt,
                    0x3B => IfTest::Ge,
                    0x3C => IfTest::Gt,
                    0x3D => IfTest::Le,
                    _ => unreachable!(),
                };
                Self::IfTestZ { kind, a: immediate_args, off: next!() as i16 }
            },
            0x44..=0x51 => {
                let kind = if opcode_byte < 0x4B {
                    Op::Get
                } else {
                    Op::Put
                };
                let kind_type = match opcode_byte {
                    0x44 | 0x4B => OpType::None,
                    0x45 | 0x4C => OpType::Wide,
                    0x46 | 0x4D => OpType::Object,
                    0x47 | 0x4E => OpType::Boolean,
                    0x48 | 0x4F => OpType::Byte,
                    0x49 | 0x50 => OpType::Char,
                    0x4A | 0x51 => OpType::Short,
                    _ => unreachable!(),
                };
                let (arr, idx) = word_to_bytes(next!());
                Self::ArrayOp { kind, kind_type, val: immediate_args, arr, idx }
            },
            0x52..=0x5F => {
                let kind = if opcode_byte < 0x59 {
                    Op::Get
                } else {
                    Op::Put
                };
                let kind_type = match opcode_byte {
                    0x52 | 0x59 => OpType::None,
                    0x53 | 0x5A => OpType::Wide,
                    0x54 | 0x5B => OpType::Object,
                    0x55 | 0x5C => OpType::Boolean,
                    0x56 | 0x5D => OpType::Byte,
                    0x57 | 0x5E => OpType::Char,
                    0x58 | 0x5F => OpType::Short,
                    _ => unreachable!(),
                };
                let (val, obj) = byte_to_nibbles(immediate_args);
                Self::InstanceOp { val, obj, idx: next!() }
            },
            0x60..=0x6D => {
                let kind = if opcode_byte < 0x67 {
                    Op::Get
                } else {
                    Op::Put
                };
                let kind_type = match opcode_byte {
                    0x60 | 0x67 => OpType::None,
                    0x61 | 0x68 => OpType::Wide,
                    0x62 | 0x69 => OpType::Object,
                    0x63 | 0x6A => OpType::Boolean,
                    0x64 | 0x6B => OpType::Byte,
                    0x65 | 0x6C => OpType::Char,
                    0x66 | 0x6D => OpType::Short,
                    _ => unreachable!(),
                };
                Self::StaticOp { val: immediate_args, idx: next!() }
            },
            0x6E..=0x72 => {
                let kind = match opcode_byte {
                    0x6E => InvokeKind::Virtual,
                    0x6F => InvokeKind::Super,
                    0x70 => InvokeKind::Direct,
                    0x71 => InvokeKind::Static,
                    0x72 => InvokeKind::Interface,
                    _ => unreachable!(),
                };
                let (argc, c) = byte_to_nibbles(immediate_args);
                let idx = next!();
                let (d, e, f, g) = word_to_nibbles(next!());
                Self::Invoke { kind, argc, idx, c, d, e, f, g }
            },
            0x74..=0x78 => {
                let kind = match opcode_byte {
                    0x74 => InvokeKind::Virtual,
                    0x75 => InvokeKind::Super,
                    0x76 => InvokeKind::Direct,
                    0x77 => InvokeKind::Static,
                    0x78 => InvokeKind::Interface,
                    _ => unreachable!(),
                };
                Self::InvokeRange { kind, argc: immediate_args, idx: next!(), first: next!() }
            },
            0x7B..=0x8F => {
                let kind = match opcode_byte {
                    0x7B => UnopKind::NegInt,
                    0x7C => UnopKind::NotInt,
                    0x7D => UnopKind::NegLong,
                    0x7E => UnopKind::NotLong,
                    0x7F => UnopKind::NegFloat,
                    0x80 => UnopKind::NegDouble,
                    0x81 => UnopKind::IntToLong,
                    0x82 => UnopKind::IntToFloat,
                    0x83 => UnopKind::IntToDouble,
                    0x84 => UnopKind::LongToInt,
                    0x85 => UnopKind::LongToFloat,
                    0x86 => UnopKind::LongToDouble,
                    0x87 => UnopKind::FloatToInt,
                    0x88 => UnopKind::FloatToLong,
                    0x89 => UnopKind::FloatToDouble,
                    0x8A => UnopKind::DoubleToInt,
                    0x8B => UnopKind::DoubleToLong,
                    0x8C => UnopKind::DoubleToFloat,
                    0x8D => UnopKind::IntToByte,
                    0x8E => UnopKind::IntToChar,
                    0x8F => UnopKind::IntToShort,
                    _ => unreachable!(),
                };
                let (dst, src) = byte_to_nibbles(immediate_args);
                Self::Unop { kind, dst, src }
            },
            0x90..=0xAF => {
                let kind = match opcode_byte {
                    0x90 => BinopKind::AddInt,
                    0x91 => BinopKind::SubInt,
                    0x92 => BinopKind::MulInt,
                    0x93 => BinopKind::DivInt,
                    0x94 => BinopKind::RemInt,
                    0x95 => BinopKind::AndInt,
                    0x96 => BinopKind::OrInt,
                    0x97 => BinopKind::XorInt,
                    0x98 => BinopKind::ShlInt,
                    0x99 => BinopKind::ShrInt,
                    0x9A => BinopKind::UshrInt,
                    0x9B => BinopKind::AddLong,
                    0x9C => BinopKind::SubLong,
                    0x9D => BinopKind::MulLong,
                    0x9E => BinopKind::DivLong,
                    0x9F => BinopKind::RemLong,
                    0xA0 => BinopKind::AndLong,
                    0xA1 => BinopKind::OrLong,
                    0xA2 => BinopKind::XorLong,
                    0xA3 => BinopKind::ShlLong,
                    0xA4 => BinopKind::ShrLong,
                    0xA5 => BinopKind::UshrLong,
                    0xA6 => BinopKind::AddFloat,
                    0xA7 => BinopKind::SubFloat,
                    0xA8 => BinopKind::MulFloat,
                    0xA9 => BinopKind::DivFloat,
                    0xAA => BinopKind::RemFloat,
                    0xAB => BinopKind::AddDouble,
                    0xAC => BinopKind::SubDouble,
                    0xAD => BinopKind::MulDouble,
                    0xAE => BinopKind::DivDouble,
                    0xAF => BinopKind::RemDouble,
                    _ => unreachable!(),
                };
                let (src0, src1) = word_to_bytes(next!());
                Self::Binop { kind, dst: immediate_args, src0, src1 }
            },
            0xB0..=0xCF => {
                let kind = match opcode_byte {
                    0xB0 => BinopKind::AddInt,
                    0xB1 => BinopKind::SubInt,
                    0xB2 => BinopKind::MulInt,
                    0xB3 => BinopKind::DivInt,
                    0xB4 => BinopKind::RemInt,
                    0xB5 => BinopKind::AndInt,
                    0xB6 => BinopKind::OrInt,
                    0xB7 => BinopKind::XorInt,
                    0xB8 => BinopKind::ShlInt,
                    0xB9 => BinopKind::ShrInt,
                    0xBA => BinopKind::UshrInt,
                    0xBB => BinopKind::AddLong,
                    0xBC => BinopKind::SubLong,
                    0xBD => BinopKind::MulLong,
                    0xBE => BinopKind::DivLong,
                    0xBF => BinopKind::RemLong,
                    0xC0 => BinopKind::AndLong,
                    0xC1 => BinopKind::OrLong,
                    0xC2 => BinopKind::XorLong,
                    0xC3 => BinopKind::ShlLong,
                    0xC4 => BinopKind::ShrLong,
                    0xC5 => BinopKind::UshrLong,
                    0xC6 => BinopKind::AddFloat,
                    0xC7 => BinopKind::SubFloat,
                    0xC8 => BinopKind::MulFloat,
                    0xC9 => BinopKind::DivFloat,
                    0xCA => BinopKind::RemFloat,
                    0xCB => BinopKind::AddDouble,
                    0xCC => BinopKind::SubDouble,
                    0xCD => BinopKind::MulDouble,
                    0xCE => BinopKind::DivDouble,
                    0xCF => BinopKind::RemDouble,
                    _ => unreachable!(),
                };
                let (dst, src) = byte_to_nibbles(immediate_args);
                Self::Binop2Addr { kind, dst, src }
            },
            0xD0..=0xE2 => {
                let kind = match opcode_byte {
                    0xD0 | 0xD8 => BinopKind::AddInt,
                    0xD1 | 0xD9 => BinopKind::SubInt,
                    0xD2 | 0xDA => BinopKind::MulInt,
                    0xD3 | 0xDB => BinopKind::DivInt,
                    0xD4 | 0xDC => BinopKind::RemInt,
                    0xD5 | 0xDD => BinopKind::AndInt,
                    0xD6 | 0xDE => BinopKind::OrInt,
                    0xD7 | 0xDF => BinopKind::XorInt,
                    0xE0 => BinopKind::ShlInt,
                    0xE1 => BinopKind::ShrInt,
                    0xE2 => BinopKind::UshrInt,
                    _ => unreachable!(),
                };
                if opcode_byte < 0xD8 {
                    let (dst, src) = byte_to_nibbles(immediate_args);
                    Self::BinopLit { kind, dst, src, lit: next!() as i16 }
                } else {
                    let (src, lit) = word_to_bytes(next!());
                    // TODO: Check if this is correct
                    Self::BinopLit { kind, dst: immediate_args, src, lit: lit as i16 }
                }
            },
            0xFA => {
                let (argc, recv) = byte_to_nibbles(immediate_args);
                let (d, e, f, g) = word_to_nibbles(next!());
                let midx = next!();
                let pidx = next!();
                Self::InvokePolymorphic { argc, recv, midx, d, e, f, g, pidx }
            },
            0xFB => {
                // TODO: Check if this is correct
                let recv = next!();
                let midx = next!();
                let pidx = next!();
                Self::InvokePolymorphicRange { argc: immediate_args, midx, recv, pidx }
            },
            0xFC => {
                // TODO: Check if this is correct
                let (argc, c) = byte_to_nibbles(immediate_args);
                let (d, e, f, g) = word_to_nibbles(next!());
                let idx = next!();
                Self::InvokeCustom { argc, idx, c, d, e, f, g }
            },
            0xFD => Self::InvokeCustomRange { argc: immediate_args, idx: next!(), first: next!() },
            0xFE => Self::ConstMethodHandle {
                dst: immediate_args,
                idx: next!(),
            },
            0xFF => Self::ConstMethodType {
                dst: immediate_args,
                idx: next!(),
            },
            other => return Err(InstructionError::BadOpcode(other)),
        };
        Ok(Some(instruction))
    }
}
