// op_code

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OpCode {
    PushInt8 = 0x00,
    PushInt16 = 0x01,
    PushInt32 = 0x02,
    PushInt64 = 0x03,
    PushInt128 = 0x04,
    PushInt256 = 0x05,
    PushTrue = 0x08,
    PushFalse = 0x09,
    PushA = 0x0A,
    PushNull = 0x0B,
    PushData1 = 0x0C,
    PushData2 = 0x0D,
    PushData4 = 0x0E,
    PushM1 = 0x0F,
    Push0 = 0x10,
    Push1 = 0x11,
    Push2 = 0x12,
    Push3 = 0x13,
    Push4 = 0x14,
    Push5 = 0x15,
    Push6 = 0x16,
    Push7 = 0x17,
    Push8 = 0x18,
    Push9 = 0x19,
    Push10 = 0x1A,
    Push11 = 0x1B,
    Push12 = 0x1C,
    Push13 = 0x1D,
    Push14 = 0x1E,
    Push15 = 0x1F,
    Push16 = 0x20,

    Nop = 0x21,
    Jmp = 0x22,
    JmpL = 0x23,
    JmpIf = 0x24,
    JmpIfL = 0x25,
    JmpIfNot = 0x26,
    JmpIfNotL = 0x27,
    JmpEq = 0x28,
    JmpEqL = 0x29,
    JmpNe = 0x2A,
    JmpNeL = 0x2B,
    JmpGt = 0x2C,
    JmpGtL = 0x2D,
    JmpGe = 0x2E,
    JmpGeL = 0x2F,
    JmpLt = 0x30,
    JmpLtL = 0x31,
    JmpLe = 0x32,
    JmpLeL = 0x33,
    Call = 0x34,
    CallL = 0x35,
    CallA = 0x36,
    CallT = 0x37,
    Abort = 0x38,
    Assert = 0x39,
    Throw = 0x3A,
    Try = 0x3B,
    TryL = 0x3C,
    EndTry = 0x3D,
    EndTryL = 0x3E,
    EndFinally = 0x3F,
    Ret = 0x40,
    Syscall = 0x41,

    Depth = 0x43,
    Drop = 0x45,
    Nip = 0x46,
    Xdrop = 0x48,
    Clear = 0x49,
    Dup = 0x4A,
    Over = 0x4B,
    Pick = 0x4D,
    Tuck = 0x4E,
    Swap = 0x50,
    Rot = 0x51,
    Roll = 0x52,
    Reverse3 = 0x53,
    Reverse4 = 0x54,
    ReverseN = 0x55,

    InitSSLot = 0x56,
    InitSlot = 0x57,
    LdSFLd0 = 0x58,
    LdSFLd1 = 0x59,
    LdSFLd2 = 0x5A,
    LdSFLd3 = 0x5B,
    LdSFLd4 = 0x5C,
    LdSFLd5 = 0x5D,
    LdSFLd6 = 0x5E,
    LdSFLd = 0x5F,
    StSFLd0 = 0x60,
    StSFLd1 = 0x61,
    StSFLd2 = 0x62,
    StSFLd3 = 0x63,
    StSFLd4 = 0x64,
    StSFLd5 = 0x65,
    StSFLd6 = 0x66,
    StSFLd = 0x67,
    LdLoc0 = 0x68,
    LdLoc1 = 0x69,
    LdLoc2 = 0x6A,
    LdLoc3 = 0x6B,
    LdLoc4 = 0x6C,
    LdLoc5 = 0x6D,
    LdLoc6 = 0x6E,
    LdLoc = 0x6F,
    StLoc0 = 0x70,
    StLoc1 = 0x71,
    StLoc2 = 0x72,
    StLoc3 = 0x73,
    StLoc4 = 0x74,
    StLoc5 = 0x75,
    StLoc6 = 0x76,
    StLoc = 0x77,
    LdArg0 = 0x78,
    LdArg1 = 0x79,
    LdArg2 = 0x7A,
    LdArg3 = 0x7B,
    LdArg4 = 0x7C,
    LdArg5 = 0x7D,
    LdArg6 = 0x7E,
    LdArg = 0x7F,
    StArg0 = 0x80,
    StArg1 = 0x81,
    StArg2 = 0x82,
    StArg3 = 0x83,
    StArg4 = 0x84,
    StArg5 = 0x85,
    StArg6 = 0x86,
    StArg = 0x87,

    NewBuffer = 0x88,
    MemCpy = 0x89,
    Cat = 0x8B,
    Substr = 0x8C,
    Left = 0x8D,
    Right = 0x8E,

    Invert = 0x90,
    And = 0x91,
    Or = 0x92,
    Xor = 0x93,
    Equal = 0x97,
    NotEqual = 0x98,

    Sign = 0x99,
    Abs = 0x9A,
    Negate = 0x9B,
    Inc = 0x9C,
    Dec = 0x9D,
    Add = 0x9E,
    Sub = 0x9F,
    Mul = 0xA0,
    Div = 0xA1,
    Mod = 0xA2,
    Pow = 0xA3,
    Sqrt = 0xA4,
    ModMul = 0xA5,
    ModPow = 0xA6,
    Shl = 0xA8,
    Shr = 0xA9,
    Not = 0xAA,
    BoolAnd = 0xAB,
    BoolOr = 0xAC,
    Nz = 0xB1,
    NumEqual = 0xB3,
    NumNotEqual = 0xB4,
    Lt = 0xB5,
    Le = 0xB6,
    Gt = 0xB7,
    Ge = 0xB8,
    Min = 0xB9,
    Max = 0xBA,
    Within = 0xBB,

    PackMap = 0xBE,
    PackStruct = 0xBF,
    Pack = 0xC0,
    Unpack = 0xC1,
    NewArray0 = 0xC2,
    NewArray = 0xC3,
    NewArrayT = 0xC4,
    NewStruct0 = 0xC5,
    NewStruct = 0xC6,
    NewMap = 0xC8,
    Size = 0xCA,
    HasKey = 0xCB,
    Keys = 0xCC,
    Values = 0xCD,
    PickItem = 0xCE,
    Append = 0xCF,
    SetItem = 0xD0,
    ReverseItems = 0xD1,
    Remove = 0xD2,
    ClearItems = 0xD3,
    PopItem = 0xD4,

    IsNull = 0xD8,
    IsType = 0xD9,
    Convert = 0xDB,

    AbortMsg = 0xE0,
    AssertMsg = 0xE1,
}


impl OpCode {
    pub fn price(self) -> u32 {
        match self {
            OpCode::PushInt8 |
            OpCode::PushInt16 |
            OpCode::PushInt32 |
            OpCode::PushInt64 |
            OpCode::PushNull |
            OpCode::PushM1 |
            OpCode::Push0 |
            OpCode::Push1 |
            OpCode::Push2 |
            OpCode::Push3 |
            OpCode::Push4 |
            OpCode::Push5 |
            OpCode::Push6 |
            OpCode::Push7 |
            OpCode::Push8 |
            OpCode::Push9 |
            OpCode::Push10 |
            OpCode::Push11 |
            OpCode::Push12 |
            OpCode::Push13 |
            OpCode::Push14 |
            OpCode::Push15 |
            OpCode::Push16 |
            OpCode::Nop |
            OpCode::Assert => 1,
            OpCode::PushInt128 |
            OpCode::PushInt256 |
            OpCode::PushA |
            OpCode::Try |
            OpCode::Sign |
            OpCode::Abs |
            OpCode::Negate |
            OpCode::Inc |
            OpCode::Dec |
            OpCode::Not |
            OpCode::Nz |
            OpCode::Size => 1 << 2,
            OpCode::PushData1 |
            OpCode::And |
            OpCode::Or |
            OpCode::Xor |
            OpCode::Add |
            OpCode::Sub |
            OpCode::Mul |
            OpCode::Div |
            OpCode::Mod |
            OpCode::Shl |
            OpCode::Shr |
            OpCode::BoolAnd |
            OpCode::BoolOr |
            OpCode::NumEqual |
            OpCode::NumNotEqual |
            OpCode::Lt |
            OpCode::Le |
            OpCode::Gt |
            OpCode::Ge |
            OpCode::Min |
            OpCode::Max |
            OpCode::Within |
            OpCode::NewMap => 1 << 3,
            OpCode::Xdrop |
            OpCode::Clear |
            OpCode::Roll |
            OpCode::ReverseN |
            OpCode::InitSSLot |
            OpCode::NewArray0 |
            OpCode::NewStruct0 |
            OpCode::Keys |
            OpCode::Remove |
            OpCode::ClearItems => 1 << 4,
            OpCode::Equal |
            OpCode::NotEqual |
            OpCode::ModMul => 1 << 5,
            OpCode::InitSlot |
            OpCode::Pow |
            OpCode::HasKey |
            OpCode::PickItem => 1 << 6,
            OpCode::NewBuffer => 1 << 8,
            OpCode::PushData2 |
            OpCode::Call |
            OpCode::CallL |
            OpCode::CallA |
            OpCode::Throw |
            OpCode::NewArray |
            OpCode::NewArrayT |
            OpCode::NewStruct => 1 << 9,
            OpCode::MemCpy |
            OpCode::Cat |
            OpCode::Substr |
            OpCode::Left |
            OpCode::Right |
            OpCode::Sqrt |
            OpCode::ModPow |
            OpCode::PackMap |
            OpCode::PackStruct |
            OpCode::Pack |
            OpCode::Unpack => 1 << 11,
            OpCode::PushData4 => 1 << 12,
            OpCode::Values |
            OpCode::Append |
            OpCode::SetItem |
            OpCode::ReverseItems |
            OpCode::Convert => 1 << 13,
            OpCode::CallT => 1 << 15,
            OpCode::Abort |
            OpCode::Ret |
            OpCode::Syscall => 0,
            _ => 1 << 1,
        }
    }
}