#[derive(Debug)]
pub enum Instruction {
    Nop,
    Move {
        dst: u8,
        src: u8,
    },
    MoveFrom16 {
        dst: u8,
        src: u16,
    },
    Move16 {
        dst: u16,
        src: u16,
    },
    MoveWide {
        dst: u8,
        src: u8,
    },
    MoveWideFrom16 {
        dst: u8,
        src: u16,
    },
    MoveWide16 {
        dst: u16,
        src: u16,
    },
    MoveObject {
        dst: u8,
        src: u8,
    },
    MoveObjectFrom16 {
        dst: u8,
        src: u16,
    },
    MoveObject16 {
        dst: u16,
        src: u16,
    },
    MoveResult {
        dst: u8,
    },
    MoveResultWide {
        dst: u8,
    },
    MoveResultObject {
        dst: u8,
    },
    MoveException {
        dst: u8,
    },
    ReturnVoid,
    Return {
        src: u8,
    },
    ReturnWide {
        src: u8,
    },
    ReturnObject {
        src: u8,
    },
    Const4 {
        dst: u8,
        value: i8,
    },
    Const16 {
        dst: u8,
        value: i16,
    },
    Const {
        dst: u8,
        value: u32,
    },
    ConstHigh16 {
        dst: u8,
        value: i16,
    },
    ConstWide16 {
        dst: u8,
        value: i16,
    },
    ConstWide32 {
        dst: u8,
        value: i32,
    },
    ConstWide {
        dst: u8,
        value: u64,
    },
    ConstWideHigh16 {
        dst: u8,
        value: i16,
    },
    ConstString {
        dst: u8,
        idx: u16,
    },
    ConstStringJumbo {
        dst: u8,
        idx: u32,
    },
    ConstClass {
        dst: u8,
        idx: u16,
    },
    MonitorEnter {
        obj: u8,
    },
    MonitorExit {
        obj: u8,
    },
    CheckCast {
        obj: u8,
        idx: u16,
    },
    InstanceOf {
        dst: u8,
        obj: u8,
        idx: u16,
    },
    ArrayLength {
        dst: u8,
        obj: u8,
    },
    NewInstance {
        dst: u8,
        idx: u16,
    },
    NewArray {
        dst: u8,
        size: u8,
        idx: u16,
    },
    FilledNewArray {
        size: u8,
        idx: u16,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        g: u8,
    },
    FilledNewArrayRange {
        size: u8,
        idx: u16,
        first: u16,
    },
    FillArrayData {
        arr: u8,
        off: u32,
    },
    Throw {
        off: u8,
    },
    Goto {
        off: u8,
    },
    Goto16 {
        off: i16,
    },
    Goto32 {
        off: i32,
    },
    PackedSwitch {
        reg: u8,
        off: i32,
    },
    SparseSwitch {
        reg: u8,
        off: i32,
    },
    CmpKind {
        kind: CmpKind,
        dst: u8,
        src0: u8,
        src1: u8,
    },
    IfTest {
        kind: IfTest,
        a: u8,
        b: u8,
        off: i16,
    },
    IfTestZ {
        kind: IfTest,
        a: u8,
        off: i16,
    },
    ArrayOp {
        kind: Op,
        kind_type: OpType,
        val: u8,
        arr: u8,
        idx: u8,
    },
    InstanceOp {
        val: u8,
        obj: u8,
        idx: u16,
    },
    StaticOp {
        val: u8,
        idx: u16,
    },
    Invoke {
        kind: InvokeKind,
        argc: u8,
        idx: u16,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        g: u8,
    },
    InvokeRange {
        kind: InvokeKind,
        argc: u8,
        idx: u16,
        first: u16,
    },
    Unop {
        kind: UnopKind,
        dst: u8,
        src: u8,
    },
    Binop {
        kind: BinopKind,
        dst: u8,
        src0: u8,
        src1: u8,
    },
    Binop2Addr {
        kind: BinopKind,
        dst: u8,
        src: u8,
    },
    BinopLit {
        kind: BinopKind,
        dst: u8,
        src: u8,
        lit: i16,
    },
    InvokePolymorphic {
        argc: u8,
        midx: u16,
        recv: u8,
        pidx: u16,
        d: u8,
        e: u8,
        f: u8,
        g: u8,
    },
    InvokePolymorphicRange {
        argc: u8,
        midx: u16,
        recv: u16,
        pidx: u16,
    },
    InvokeCustom {
        argc: u8,
        idx: u16,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        g: u8,
    },
    InvokeCustomRange {
        argc: u8,
        idx: u16,
        first: u16,
    },
    ConstMethodHandle {
        dst: u8,
        idx: u16,
    },
    ConstMethodType {
        dst: u8,
        idx: u16,
    },
}

#[derive(Debug)]
pub enum CmpKind {
    LtFloat,
    GtFloat,
    LtDouble,
    GtDouble,
    Long,
}

#[derive(Debug)]
pub enum IfTest {
    Eq,
    Ne,
    Lt,
    Ge,
    Gt,
    Le,
}

#[derive(Debug)]
pub enum Op {
    Get,
    Put,
}

#[derive(Debug)]
pub enum OpType {
    None,
    Wide,
    Object,
    Boolean,
    Byte,
    Char,
    Short,
}

#[derive(Debug)]
pub enum InvokeKind {
    Virtual,
    Super,
    Direct,
    Static,
    Interface,
}

#[derive(Debug)]
pub enum UnopKind {
    NegInt,
    NotInt,
    NegLong,
    NotLong,
    NegFloat,
    NegDouble,
    IntToLong,
    IntToFloat,
    IntToDouble,
    LongToInt,
    LongToFloat,
    LongToDouble,
    FloatToInt,
    FloatToLong,
    FloatToDouble,
    DoubleToInt,
    DoubleToLong,
    DoubleToFloat,
    IntToByte,
    IntToChar,
    IntToShort,
}

#[derive(Debug)]
pub enum BinopKind {
    AddInt,
    SubInt,
    MulInt,
    DivInt,
    RemInt,
    AndInt,
    OrInt,
    XorInt,
    ShlInt,
    ShrInt,
    UshrInt,
    AddLong,
    SubLong,
    MulLong,
    DivLong,
    RemLong,
    AndLong,
    OrLong,
    XorLong,
    ShlLong,
    ShrLong,
    UshrLong,
    AddFloat,
    SubFloat,
    MulFloat,
    DivFloat,
    RemFloat,
    AddDouble,
    SubDouble,
    MulDouble,
    DivDouble,
    RemDouble,
}
