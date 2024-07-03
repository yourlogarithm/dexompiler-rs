#[derive(Debug)]
pub struct F12x {
    pub va: u8,
    pub vb: u8,
}

#[derive(Debug)]
pub struct F11n {
    pub va: u8,
    pub literal: i8,
}

#[derive(Debug)]
pub struct F11x {
    pub va: u8,
}

#[derive(Debug)]
pub struct F10t {
    pub offset: i8,
}

#[derive(Debug)]
pub struct F20t {
    pub offset: i16,
}

#[derive(Debug)]
pub struct F20bc {
    pub va: u8,
    pub idx: u16,
}

#[derive(Debug)]
pub struct F22x {
    pub va: u8,
    pub vb: u16,
}

#[derive(Debug)]
pub struct F21t {
    pub va: u8,
    pub offset: i16,
}

#[derive(Debug)]
pub struct F21s {
    pub va: u8,
    pub literal: i16,
}

#[derive(Debug)]
pub struct F21h {
    pub va: u8,
    pub literal: i16,
}

#[derive(Debug)]
pub struct F21c {
    pub dst: u8,
    pub idx: u16,
}

#[derive(Debug)]
pub struct F23x {
    pub va: u8,
    pub vb: u8,
    pub vc: u8,
}

#[derive(Debug)]
pub struct F22b {
    pub va: u8,
    pub vb: u8,
    pub literal: i8,
}

#[derive(Debug)]
pub struct F22t {
    pub va: u8,
    pub vb: u8,
    pub offset: i16,
}

#[derive(Debug)]
pub struct F22s {
    pub va: u8,
    pub vb: u8,
    pub literal: i16,
}

#[derive(Debug)]
pub struct F22c {
    pub va: u8,
    pub vb: u8,
    pub idx: u16,
}

#[derive(Debug)]
pub struct F22cs {
    pub va: u8,
    pub vb: u8,
    pub idx: u16,
}

#[derive(Debug)]
pub struct F30t {
    pub offset: i32,
}

#[derive(Debug)]
pub struct F32x {
    pub va: u16,
    pub vb: u16,
}

#[derive(Debug)]
pub struct F31i {
    pub va: u8,
    pub literal: i32,
}

#[derive(Debug)]
pub struct F31t {
    pub va: u8,
    pub offset: i32,
}

#[derive(Debug)]
pub struct F31c {
    pub va: u8,
    pub idx: u32,
}

#[derive(Debug)]
pub struct F35c {
    pub argc: u8,
    pub args: [u8; 5],
    pub idx: u16,
}

#[derive(Debug)]
pub struct F35ms {
    pub va: u8,
    pub args: [u8; 5],
    pub idx: u16,
}

#[derive(Debug)]
pub struct F35mi {
    pub va: u8,
    pub args: [u8; 5],
    pub idx: u16,
}

#[derive(Debug)]
pub struct F3rc {
    pub argc: u8,
    pub reg: u16,
    pub idx: u16,
}

#[derive(Debug)]
pub struct F3rms {
    pub va: u8,
    pub reg: u16,
    pub idx: u16,
}

#[derive(Debug)]
pub struct F3rmi {
    pub va: u8,
    pub reg: u16,
    pub idx: u16,
}

#[derive(Debug)]
pub struct F45cc {
    pub argc: u8,
    pub vg: u8,
    pub args: [u8; 5],
    pub meth: u16,
    pub proto: u16,
}

#[derive(Debug)]
pub struct F4rcc {
    pub argc: u8,
    pub reg: u16,
    pub meth: u16,
    pub proto: u16,
}

#[derive(Debug)]
pub struct F51l {
    pub va: u8,
    pub literal: i64,
}

#[derive(Debug)]
pub enum Format {
    F10x,
    F12x(F12x),
    F11n(F11n),
    F11x(F11x),
    F10t(F10t),
    F20t(F20t),
    F20bc(F20bc),
    F22x(F22x),
    F21t(F21t),
    F21s(F21s),
    F21h(F21h),
    F21c(F21c),
    F23x(F23x),
    F22b(F22b),
    F22t(F22t),
    F22s(F22s),
    F22c(F22c),
    F22cs(F22cs),
    F30t(F30t),
    F32x(F32x),
    F31i(F31i),
    F31t(F31t),
    F31c(F31c),
    F35c(F35c),
    F35ms(F35ms),
    F35mi(F35mi),
    F3rc(F3rc),
    F3rms(F3rms),
    F3rmi(F3rmi),
    F45cc(F45cc),
    F4rcc(F4rcc),
    F51l(F51l),
}

impl Format {
    pub fn len(&self) -> usize {
        match self {
            Format::F10x => 1,
            Format::F12x(_) => 1,
            Format::F11n(_) => 1,
            Format::F11x(_) => 1,
            Format::F10t(_) => 1,
            Format::F20t(_) => 2,
            Format::F20bc(_) => 2,
            Format::F22x(_) => 2,
            Format::F21t(_) => 2,
            Format::F21s(_) => 2,
            Format::F21h(_) => 2,
            Format::F21c(_) => 2,
            Format::F23x(_) => 2,
            Format::F22b(_) => 2,
            Format::F22t(_) => 2,
            Format::F22s(_) => 2,
            Format::F22c(_) => 2,
            Format::F22cs(_) => 2,
            Format::F30t(_) => 3,
            Format::F32x(_) => 3,
            Format::F31i(_) => 3,
            Format::F31t(_) => 3,
            Format::F31c(_) => 3,
            Format::F35c(_) => 3,
            Format::F35ms(_) => 3,
            Format::F35mi(_) => 3,
            Format::F3rc(_) => 3,
            Format::F3rms(_) => 3,
            Format::F3rmi(_) => 3,
            Format::F45cc(_) => 4,
            Format::F4rcc(_) => 4,
            Format::F51l(_) => 5,
        }
    }

    pub fn offset(&self) -> Option<i32> {
        match *self {
            Format::F10t(F10t { offset }) => Some(offset as i32),
            Format::F20t(F20t { offset }) => Some(offset as i32),
            Format::F21t(F21t { offset, .. }) => Some(offset as i32),
            Format::F22t(F22t { offset, .. }) => Some(offset as i32),
            Format::F30t(F30t { offset, .. }) => Some(offset),
            Format::F31t(F31t { offset, .. }) => Some(offset),
            _ => None,
        }
    }

    pub fn as_12x(&self) -> Option<&F12x> {
        match self {
            Format::F12x(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_11n(&self) -> Option<&F11n> {
        match self {
            Format::F11n(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_11x(&self) -> Option<&F11x> {
        match self {
            Format::F11x(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_10t(&self) -> Option<&F10t> {
        match self {
            Format::F10t(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_20t(&self) -> Option<&F20t> {
        match self {
            Format::F20t(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_20bc(&self) -> Option<&F20bc> {
        match self {
            Format::F20bc(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_22x(&self) -> Option<&F22x> {
        match self {
            Format::F22x(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_21t(&self) -> Option<&F21t> {
        match self {
            Format::F21t(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_21s(&self) -> Option<&F21s> {
        match self {
            Format::F21s(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_21h(&self) -> Option<&F21h> {
        match self {
            Format::F21h(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_21c(&self) -> Option<&F21c> {
        match self {
            Format::F21c(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_23x(&self) -> Option<&F23x> {
        match self {
            Format::F23x(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_22b(&self) -> Option<&F22b> {
        match self {
            Format::F22b(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_22t(&self) -> Option<&F22t> {
        match self {
            Format::F22t(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_22s(&self) -> Option<&F22s> {
        match self {
            Format::F22s(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_22c(&self) -> Option<&F22c> {
        match self {
            Format::F22c(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_22cs(&self) -> Option<&F22cs> {
        match self {
            Format::F22cs(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_30t(&self) -> Option<&F30t> {
        match self {
            Format::F30t(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_32x(&self) -> Option<&F32x> {
        match self {
            Format::F32x(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_31i(&self) -> Option<&F31i> {
        match self {
            Format::F31i(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_31t(&self) -> Option<&F31t> {
        match self {
            Format::F31t(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_31c(&self) -> Option<&F31c> {
        match self {
            Format::F31c(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_35c(&self) -> Option<&F35c> {
        match self {
            Format::F35c(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_35ms(&self) -> Option<&F35ms> {
        match self {
            Format::F35ms(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_35mi(&self) -> Option<&F35mi> {
        match self {
            Format::F35mi(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_3rc(&self) -> Option<&F3rc> {
        match self {
            Format::F3rc(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_3rms(&self) -> Option<&F3rms> {
        match self {
            Format::F3rms(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_3rmi(&self) -> Option<&F3rmi> {
        match self {
            Format::F3rmi(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_45cc(&self) -> Option<&F45cc> {
        match self {
            Format::F45cc(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_4rcc(&self) -> Option<&F4rcc> {
        match self {
            Format::F4rcc(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_51l(&self) -> Option<&F51l> {
        match self {
            Format::F51l(f) => Some(f),
            _ => None,
        }
    }
}
