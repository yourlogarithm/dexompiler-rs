use std::num::NonZeroUsize;

use bitcode::{Decode, Encode};
use num_traits::FromPrimitive;
use serde::Serialize;

use super::{errors::InstructionError, Opcode};

macro_rules! collect_tuple {
    ($u2:expr) => {
        ($u2[0], $u2[1])
    };
}

#[derive(Debug, Serialize, Encode, Decode, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub opcode: Opcode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m_idx: Option<u16>,
}

impl Instruction {
    /// Parse the current instruction and advance the iterator
    pub fn try_from_code(
        code: &[u16],
        offset: usize,
    ) -> Result<Option<(Self, usize)>, InstructionError> {
        let raw_bytecode = &code[offset..];
        if raw_bytecode.is_empty() {
            return Ok(None);
        }
        let (opcode_byte, immediate_args) = collect_tuple!(raw_bytecode[0].to_le_bytes());
        let opcode: Opcode = FromPrimitive::from_u8(opcode_byte)
            .ok_or(InstructionError::BadOpcode(offset, opcode_byte))?;
        let (length, m_idx) = match opcode_byte {
            0x0 => {
                if (1..=3).contains(&immediate_args) {
                    return Ok(None);
                }
                (1, None)
            }
            0x01
            | 0x04
            | 0x07
            | 0x0A..=0x12
            | 0x1D
            | 0x1E
            | 0x21
            | 0x27
            | 0x7B..=0x8F
            | 0xB0..=0xCF => (1, None),
            0x02
            | 0x05
            | 0x08
            | 0x13
            | 0x15
            | 0x16
            | 0x19
            | 0x1A
            | 0x1C
            | 0x1F
            | 0x20
            | 0x22
            | 0x23
            | 0x2D..=0x31
            | 0x44..=0x6D
            | 0x90..=0xAF
            | 0xD0..=0xE2
            | 0xFE
            | 0xFF => (2, None),
            0x03 | 0x06 | 0x09 | 0x14 | 0x17 | 0x1B | 0x24..=0x26 | 0xFC | 0xFD => (3, None),
            0x6e..=0x72 | 0x74..=0x78 => {
                if raw_bytecode.len() < 3 {
                    return Err(InstructionError::TooShort {
                        offset,
                        opcode,
                        expected: NonZeroUsize::new(3).unwrap(),
                        actual: NonZeroUsize::new(1).unwrap(),
                    });
                }
                (3, Some(raw_bytecode[1]))
            }
            0xFA | 0xFB => {
                if raw_bytecode.len() < 4 {
                    return Err(InstructionError::TooShort {
                        offset,
                        opcode,
                        expected: NonZeroUsize::new(4).unwrap(),
                        actual: NonZeroUsize::new(1).unwrap(),
                    });
                }
                (4, Some(raw_bytecode[1]))
            }
            0x18 => (5, None),
            0x28 => (1, None),
            0x29 => (2, None),
            0x2A => (3, None),
            0x2B | 0x2C => (3, None),
            0x32..=0x3D => (2, None),
            0x3e..=0x43 | 0x73 | 0x79..=0x7a | 0xe3..=0xf9 => {
                panic!("Unused opcode found {opcode_byte}")
            }
        };
        if length > raw_bytecode.len() {
            return Err(InstructionError::TooShort {
                offset,
                opcode,
                expected: NonZeroUsize::new(length).unwrap(),
                actual: NonZeroUsize::new(raw_bytecode.len()).ok_or(InstructionError::End)?,
            });
        }
        Ok(Some((Instruction { opcode, m_idx }, length)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;

    #[test]
    fn test_empty_bytecode() {
        let code: [u16; 0] = [];
        let result = Instruction::try_from_code(&code, 0);
        assert!(matches!(result, Ok(None)));
    }

    // Test a valid opcode that doesn't require additional bytes
    #[test]
    fn test_valid_opcode() {
        let code: [u16; 1] = [0x01];
        let result = Instruction::try_from_code(&code, 0);
        assert!(matches!(
            result,
            Ok(Some((
                Instruction {
                    opcode: Opcode::Move,
                    m_idx: None
                },
                1
            )))
        ));
    }

    // Test bad opcode case
    #[test]
    fn test_bad_opcode() {
        let code: [u16; 1] = [0x3E];
        let result = Instruction::try_from_code(&code, 0);
        assert!(matches!(result, Err(InstructionError::BadOpcode(0, 0x3E))));
    }

    // // Test a case where bytecode is too short for the given opcode
    #[test]
    fn test_too_short() {
        let code: [u16; 1] = [0x6E];
        let result = Instruction::try_from_code(&code, 0);
        if let Err(InstructionError::TooShort { offset, opcode, expected, actual }) = result {
            assert_eq!(offset, 0);
            assert_eq!(opcode, Opcode::InvokeVirtual);
            assert_eq!(expected, NonZeroUsize::new(3).unwrap());
            assert_eq!(actual, NonZeroUsize::new(1).unwrap());
        } else {
            panic!("Expected an error found {result:?}");
        }
    }

    // Test an invoke opcode with arguments
    #[test]
    fn test_invoke_args() {
        let code: [u16; 3] = [0x6E, 0x06, 0x00];
        let (inst, length) = Instruction::try_from_code(&code, 0).unwrap().unwrap();
        assert_eq!(length, 3);
        assert_eq!(
            inst,
            Instruction {
                opcode: Opcode::InvokeVirtual,
                m_idx: Some(6)
            }
        );
    }
}
