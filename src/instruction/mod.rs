mod error;
mod format;
mod opcode;

use std::{collections::HashMap, iter::Peekable};

pub use error::*;
pub use format::*;
use num_traits::FromPrimitive;
pub use opcode::Opcode;

#[derive(Debug)]
pub enum Instruction {
    Regular { op: Opcode, format: Format },
    Switch {
        kv: HashMap<i32, i32>,
        code_units: u16,
    },
    FillArrayData {
        size: u32,
        element_width: usize,
        data: Vec<Vec<u8>>
    },
}

const fn word_to_bytes(word: u16) -> (u8, u8) {
    let u2 = word.to_le_bytes();
    (u2[0], u2[1])
}

const fn word_to_nibbles(word: u16) -> (u8, u8, u8, u8) {
    let u2 = word.to_le_bytes();
    (u2[0] >> 4, u2[0] & 0x0F, u2[1] >> 4, u2[1] & 0x0F)
}

const fn byte_to_nibbles(byte: u8) -> (u8, u8) {
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
            () => {{
                let a = next!();
                let b = next!();
                (b as u32) << 16 | a as u32
            }};
        }
        macro_rules! qword {
            () => {
                {
                    let a = dword!();
                    let b = dword!();
                    (b as u64) << 32 | a as u64
                }
            }
        }
        let format = match opcode_byte {
            0x00 => match immediate_args {
                1 => {
                    let size = next!();
                    let first_key = dword!() as i32;
                    let mut kv = HashMap::with_capacity(size as usize);
                    for i in 0..size {
                        let offset = dword!() as i32;
                        kv.insert(first_key + i as i32, offset);
                    }
                    let code_units = size * 2 + 4;
                    return Ok(Some(Instruction::Switch { kv, code_units }));
                }
                2 => {
                    let size = next!();
                    let mut kv = HashMap::with_capacity(size as usize);

                    for _ in 0..size {
                        let key = dword!() as i32;
                        let value = dword!() as i32;
                        kv.insert(key, value);
                    }
                    
                    let code_units = size * 4 + 2;
                    return Ok(Some(Instruction::Switch { kv, code_units }));
                }
                3 => {
                    let element_width = next!() as usize;
                    let size = dword!();
                    let words_to_use = (size as usize * (element_width) + 1) / 2;
                    let mut words = Vec::with_capacity(words_to_use);
                    for _ in 0..words_to_use {
                        words.push(next!());
                    }
                    let bytes = words
                        .iter()
                        .flat_map(|&word| {
                            let (a, b) = word_to_bytes(word);
                            [a, b]
                        })
                        .collect::<Vec<_>>();
                    let data: Vec<_> = bytes
                        .chunks_exact(element_width)
                        .map(|chunk| chunk.to_vec())
                        .collect();
                    return Ok(Some(Instruction::FillArrayData { size, element_width, data }));
                }
                _ => Format::F10x,
            },
            0x01 => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F12x(F12x { va, vb })
            }
            0x02 => Format::F22x(F22x {
                va: immediate_args,
                vb: next!(),
            }),
            0x03 => Format::F32x(F32x {
                va: next!(),
                vb: next!(),
            }),
            0x04 => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F12x(F12x { va, vb })
            }
            0x05 => Format::F22x(F22x {
                va: immediate_args,
                vb: next!(),
            }),
            0x06 => Format::F32x(F32x {
                va: next!(),
                vb: next!(),
            }),
            0x07 => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F12x(F12x { va, vb })
            }
            0x08 => Format::F22x(F22x {
                va: immediate_args,
                vb: next!(),
            }),
            0x09 => Format::F32x(F32x {
                va: next!(),
                vb: next!(),
            }),
            0x0A..=0x0D => Format::F11x(F11x { va: immediate_args }),
            0x0E => Format::F10x,
            0x0F..=0x11 => Format::F11x(F11x { va: immediate_args }),
            0x12 => {
                let (va, literal) = byte_to_nibbles(immediate_args);
                Format::F11n(F11n {
                    va,
                    literal: literal as i8,
                })
            }
            0x13 => Format::F21s(F21s {
                va: immediate_args,
                literal: next!() as i16,
            }),
            0x14 => Format::F31i(F31i {
                va: immediate_args,
                literal: dword!() as i32,
            }),
            0x15 => Format::F21h(F21h {
                va: immediate_args,
                literal: next!() as i16,
            }),
            0x16 => Format::F21s(F21s {
                va: immediate_args,
                literal: next!() as i16,
            }),
            0x17 => Format::F31i(F31i {
                va: immediate_args,
                literal: dword!() as i32,
            }),
            0x18 => Format::F51l(F51l {
                va: immediate_args,
                literal: qword!() as i64,
            }),
            0x19 => Format::F21h(F21h {
                va: immediate_args,
                literal: next!() as i16,
            }),
            0x1A => Format::F21c(F21c {
                va: immediate_args,
                idx: next!(),
            }),
            0x1B => Format::F31c(F31c {
                va: immediate_args,
                idx: dword!(),
            }),
            0x1C => Format::F21c(F21c {
                va: immediate_args,
                idx: next!(),
            }),
            0x1D..=0x1E => Format::F11x(F11x { va: immediate_args }),
            0x1F => Format::F21c(F21c {
                va: immediate_args,
                idx: next!(),
            }),
            0x20 => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F22c(F22c {
                    va,
                    vb,
                    idx: next!(),
                })
            }
            0x21 => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F12x(F12x { va, vb })
            }
            0x22 => Format::F21c(F21c {
                va: immediate_args,
                idx: next!(),
            }),
            0x23 => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F22c(F22c {
                    va,
                    vb,
                    idx: next!(),
                })
            }
            0x24 => {
                let (va, vg) = byte_to_nibbles(immediate_args);
                let idx = next!();
                let (vf, ve, vd, vc) = word_to_nibbles(next!());
                Format::F35c(F35c {
                    va,
                    args: [vc, vd, ve, vf, vg],
                    idx,
                })
            }
            0x25 => Format::F3rc(F3rc {
                va: immediate_args,
                reg: next!(),
                idx: next!(),
            }),
            0x26 => Format::F31t(F31t {
                va: immediate_args,
                offset: dword!() as i32,
            }),
            0x27 => Format::F11x(F11x { va: immediate_args }),
            0x28 => Format::F10t(F10t {
                offset: immediate_args as i8,
            }),
            0x29 => Format::F20t(F20t {
                offset: next!() as i16,
            }),
            0x2A => Format::F30t(F30t {
                offset: dword!() as i32,
            }),
            0x2B..=0x2C => Format::F31t(F31t {
                va: immediate_args,
                offset: dword!() as i32,
            }),
            0x2D..=0x31 => {
                let (src0, src1) = word_to_bytes(next!());
                Format::F23x(F23x {
                    va: immediate_args,
                    vb: src0,
                    vc: src1,
                })
            }
            0x32..=0x37 => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F22t(F22t {
                    va,
                    vb,
                    offset: next!() as i16,
                })
            }
            0x38..=0x3D => Format::F21t(F21t {
                va: immediate_args,
                offset: next!() as i16,
            }),
            0x44..=0x51 => {
                let (vb, vc) = word_to_bytes(next!());
                Format::F23x(F23x {
                    va: immediate_args,
                    vb,
                    vc,
                })
            }
            0x52..=0x5F => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F22c(F22c {
                    va,
                    vb,
                    idx: next!(),
                })
            }
            0x60..=0x6D => Format::F21c(F21c {
                va: immediate_args,
                idx: next!(),
            }),
            0x6E..=0x72 => {
                let (va, vg) = byte_to_nibbles(immediate_args);
                let idx = next!();
                let (vf, ve, vd, vc) = word_to_nibbles(next!());
                Format::F35c(F35c {
                    va,
                    args: [vc, vd, ve, vf, vg],
                    idx,
                })
            }
            0x74..=0x78 => Format::F3rc(F3rc {
                va: immediate_args,
                reg: next!(),
                idx: next!(),
            }),
            0x7B..=0x8F => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F12x(F12x { va, vb })
            }
            0x90..=0xAF => {
                let (vb, vc) = word_to_bytes(next!());
                Format::F23x(F23x {
                    va: immediate_args,
                    vb,
                    vc,
                })
            }
            0xB0..=0xCF => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F12x(F12x { va, vb })
            }
            0xD0..=0xD7 => {
                let (va, vb) = byte_to_nibbles(immediate_args);
                Format::F22s(F22s {
                    va,
                    vb,
                    literal: next!() as i16,
                })
            }
            0xD8..=0xE2 => {
                let (vb, literal) = word_to_bytes(next!());
                Format::F22b(F22b {
                    va: immediate_args,
                    vb,
                    literal: literal as i8,
                })
            }
            0xFA => {
                let (va, vg) = byte_to_nibbles(immediate_args);
                let meth = next!();
                let (vf, ve, vd, vc) = word_to_nibbles(next!());
                let proto = next!();
                Format::F45cc(F45cc {
                    va,
                    vg,
                    args: [vc, vd, ve, vf, vg],
                    meth,
                    proto,
                })
            }
            0xFB => Format::F4rcc(F4rcc {
                va: immediate_args,
                reg: next!(),
                meth: next!(),
                proto: next!(),
            }),
            0xFC => {
                let (va, vg) = byte_to_nibbles(immediate_args);
                let idx = next!();
                let (vf, ve, vd, vc) = word_to_nibbles(next!());
                Format::F35c(F35c {
                    va,
                    args: [vc, vd, ve, vf, vg],
                    idx,
                })
            }
            0xFD => Format::F3rc(F3rc {
                va: immediate_args,
                reg: next!(),
                idx: next!(),
            }),
            0xFE..=0xFF => Format::F21c(F21c {
                va: immediate_args,
                idx: next!(),
            }),
            _ => return Err(InstructionError::BadOpcode(opcode_byte)),
        };
        let op =
            FromPrimitive::from_u8(opcode_byte).ok_or(InstructionError::BadOpcode(opcode_byte))?;
        Ok(Some(Self::Regular { op, format }))
    }
}
