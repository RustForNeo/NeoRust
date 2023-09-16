// op_code

use std::fmt::{Display, Formatter};

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

impl Display for OpCode{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::PushInt8 => write!(f, "PushInt8"),
            OpCode::PushInt16 => write!(f, "PushInt16"),
            OpCode::PushInt32 => write!(f, "PushInt32"),
            OpCode::PushInt64 => write!(f, "PushInt64"),
            OpCode::PushInt128 => write!(f, "PushInt128"),
            OpCode::PushInt256 => write!(f, "PushInt256"),
            OpCode::PushTrue => write!(f, "PushTrue"),
            OpCode::PushFalse => write!(f, "PushFalse"),
            OpCode::PushA => write!(f, "PushA"),
            OpCode::PushNull => write!(f, "PushNull"),
            OpCode::PushData1 => write!(f, "PushData1"),
            OpCode::PushData2 => write!(f, "PushData2"),
            OpCode::PushData4 => write!(f, "PushData4"),
            OpCode::PushM1 => write!(f, "PushM1"),
            OpCode::Push0 => write!(f, "Push0"),
            OpCode::Push1 => write!(f, "Push1"),
            OpCode::Push2 => write!(f, "Push2"),
            OpCode::Push3 => write!(f, "Push3"),
            OpCode::Push4 => write!(f, "Push4"),
            OpCode::Push5 => write!(f, "Push5"),
            OpCode::Push6 => write!(f, "Push6"),
            OpCode::Push7 => write!(f, "Push7"),
            OpCode::Push8 => write!(f, "Push8"),
            OpCode::Push9 => write!(f, "Push9"),
            OpCode::Push10 => write!(f, "Push10"),
            OpCode::Push11 => write!(f, "Push11"),
            OpCode::Push12 => write!(f, "Push12"),
            OpCode::Push13 => write!(f, "Push13"),
            OpCode::Push14 => write!(f, "Push14"),
            OpCode::Push15 => write!(f, "Push15"),
            OpCode::Push16 => write!(f, "Push16"),
            OpCode::Nop => write!(f, "Nop"),
            OpCode::Jmp => write!(f, "Jmp"),
            OpCode::JmpL => write!(f, "JmpL"),
            OpCode::JmpIf => write!(f, "JmpIf"),
            OpCode::JmpIfL => write!(f, "JmpIfL"),
            OpCode::JmpIfNot => write!(f, "JmpIfNot"),
            OpCode::JmpIfNotL => write!(f, "JmpIfNotL"),
            OpCode::JmpEq => write!(f, "JmpEq"),
            OpCode::JmpEqL => write!(f, "JmpEqL"),
            OpCode::JmpNe => write!(f, "JmpNe"),
            OpCode::JmpNeL => write!(f, "JmpNeL"),
            OpCode::JmpGt => write!(f, "JmpGt"),
            OpCode::JmpGtL => write!(f, "JmpGtL"),
            OpCode::JmpGe => write!(f, "JmpGe"),
            OpCode::JmpGeL => write!(f, "JmpGeL"),
            OpCode::JmpLt => write!(f, "JmpLt"),
            OpCode::JmpLtL => write!(f, "JmpLtL"),
            OpCode::JmpLe => write!(f, "JmpLe"),
            OpCode::JmpLeL => write!(f, "JmpLeL"),
            OpCode::Call => write!(f, "Call"),
            OpCode::CallL => write!(f, "CallL"),
            OpCode::CallA => write!(f, "CallA"),
            OpCode::CallT => write!(f, "CallT"),
            OpCode::Abort => write!(f, "Abort"),
            OpCode::Assert => write!(f, "Assert"),
            OpCode::Throw => write!(f, "Throw"),
            OpCode::Try => write!(f, "Try"),
            OpCode::TryL => write!(f, "TryL"),
            OpCode::EndTry => write!(f, "EndTry"),
            OpCode::EndTryL => write!(f, "EndTryL"),
            OpCode::EndFinally => write!(f, "EndFinally"),
            OpCode::Ret => write!(f, "Ret"),
            OpCode::Syscall => write!(f, "Syscall"),
            OpCode::Depth => write!(f, "Depth"),
            OpCode::Drop => write!(f, "Drop"),
            OpCode::Nip => write!(f, "Nip"),
            OpCode::Xdrop => write!(f, "Xdrop"),
            OpCode::Clear => write!(f, "Clear"),
            OpCode::Dup => write!(f, "Dup"),
            OpCode::Over => write!(f, "Over"),
            OpCode::Pick => write!(f, "Pick"),
            OpCode::Tuck => write!(f, "Tuck"),
            OpCode::Swap => write!(f, "Swap"),
            OpCode::Rot => write!(f, "Rot"),
            OpCode::Roll => write!(f, "Roll"),
            OpCode::Reverse3 => write!(f, "Reverse3"),
            OpCode::Reverse4 => write!(f, "Reverse4"),
            OpCode::ReverseN => write!(f, "ReverseN"),
            OpCode::InitSSLot => write!(f, "InitSSLot"),
            OpCode::InitSlot => write!(f, "InitSlot"),
            OpCode::LdSFLd0 => write!(f, "LdSFLd0"),
            OpCode::LdSFLd1 => write!(f, "LdSFLd1"),
            OpCode::LdSFLd2 => write!(f, "LdSFLd2"),
            OpCode::LdSFLd3 => write!(f, "LdSFLd3"),
            OpCode::LdSFLd4 => write!(f, "LdSFLd4"),
            OpCode::LdSFLd5 => write!(f, "LdSFLd5"),
            OpCode::LdSFLd6 => write!(f, "LdSFLd6"),
            OpCode::LdSFLd => write!(f, "LdSFLd"),
            OpCode::StSFLd0 => write!(f, "StSFLd0"),
            OpCode::StSFLd1 => write!(f, "StSFLd1"),
            OpCode::StSFLd2 => write!(f, "StSFLd2"),
            OpCode::StSFLd3 => write!(f, "StSFLd3"),
            OpCode::StSFLd4 => write!(f, "StSFLd4"),
            OpCode::StSFLd5 => write!(f, "StSFLd5"),
            OpCode::StSFLd6 => write!(f, "StSFLd6"),
            OpCode::StSFLd => write!(f, "StSFLd"),
            OpCode::LdLoc0 => write!(f, "LdLoc0"),
            OpCode::LdLoc1 => write!(f, "LdLoc1"),
            OpCode::LdLoc2 => write!(f, "LdLoc2"),
            OpCode::LdLoc3 => write!(f, "LdLoc3"),
            OpCode::LdLoc4 => write!(f, "LdLoc4"),
            OpCode::LdLoc5 => write!(f, "LdLoc5"),
            OpCode::LdLoc6 => write!(f, "LdLoc6"),
            OpCode::LdLoc => write!(f, "LdLoc"),
            OpCode::StLoc0 => write!(f, "StLoc0"),
            OpCode::StLoc1 => write!(f, "StLoc1"),
            OpCode::StLoc2 => write!(f, "StLoc2"),
            OpCode::StLoc3 => write!(f, "StLoc3"),
            OpCode::StLoc4 => write!(f, "StLoc4"),
            OpCode::StLoc5 => write!(f, "StLoc5"),
            OpCode::StLoc6 => write!(f, "StLoc6"),
            OpCode::StLoc => write!(f, "StLoc"),
            OpCode::LdArg0 => write!(f, "LdArg0"),
            OpCode::LdArg1 => write!(f, "LdArg1"),
            OpCode::LdArg2 => write!(f, "LdArg2"),
            OpCode::LdArg3 => write!(f, "LdArg3"),
            OpCode::LdArg4 => write!(f, "LdArg4"),
            OpCode::LdArg5 => write!(f, "LdArg5"),
            OpCode::LdArg6 => write!(f, "LdArg6"),
            OpCode::LdArg => write!(f, "LdArg"),
            OpCode::StArg0 => write!(f, "StArg0"),
            OpCode::StArg1 => write!(f, "StArg1"),
            OpCode::StArg2 => write!(f, "StArg2"),
            OpCode::StArg3 => write!(f, "StArg3"),
            OpCode::StArg4 => write!(f, "StArg4"),
            OpCode::StArg5 => write!(f, "StArg5"),
            OpCode::StArg6 => write!(f, "StArg6"),
            OpCode::StArg => write!(f, "StArg"),
            OpCode::NewBuffer => write!(f, "NewBuffer"),
            OpCode::MemCpy => write!(f, "MemCpy"),
            OpCode::Cat => write!(f, "Cat"),
            OpCode::Substr => write!(f, "Substr"),
            OpCode::Left => write!(f, "Left"),
            OpCode::Right => write!(f, "Right"),
            OpCode::Invert => write!(f, "Invert"),
            OpCode::And => write!(f, "And"),
            OpCode::Or => write!(f, "Or"),
            OpCode::Xor => write!(f, "Xor"),
            OpCode::Equal => write!(f, "Equal"),
            OpCode::NotEqual => write!(f, "NotEqual"),
            OpCode::Sign => write!(f, "Sign"),
            OpCode::Abs => write!(f, "Abs"),
            OpCode::Negate => write!(f, "Negate"),
            OpCode::Inc => write!(f, "Inc"),
            OpCode::Dec => write!(f, "Dec"),
            OpCode::Add => write!(f, "Add"),
            OpCode::Sub => write!(f, "Sub"),
            OpCode::Mul => write!(f, "Mul"),
            OpCode::Div => write!(f, "Div"),
            OpCode::Mod => write!(f, "Mod"),
            OpCode::Pow => write!(f, "Pow"),
            OpCode::Sqrt => write!(f, "Sqrt"),
            OpCode::ModMul => write!(f, "ModMul"),
            OpCode::ModPow => write!(f, "ModPow"),
            OpCode::Shl => write!(f, "Shl"),
            OpCode::Shr => write!(f, "Shr"),
            OpCode::Not => write!(f, "Not"),
            OpCode::BoolAnd => write!(f, "BoolAnd"),
            OpCode::BoolOr => write!(f, "BoolOr"),
            OpCode::Nz => write!(f, "Nz"),
            OpCode::NumEqual => write!(f, "NumEqual"),
            OpCode::NumNotEqual => write!(f, "NumNotEqual"),
            OpCode::Lt => write!(f, "Lt"),
            OpCode::Le => write!(f, "Le"),
            OpCode::Gt => write!(f, "Gt"),
            OpCode::Ge => write!(f, "Ge"),
            OpCode::Min => write!(f, "Min"),
            OpCode::Max => write!(f, "Max"),
            OpCode::Within => write!(f, "Within"),
            OpCode::PackMap => write!(f, "PackMap"),
            OpCode::PackStruct => write!(f, "PackStruct"),
            OpCode::Pack => write!(f, "Pack"),
            OpCode::Unpack => write!(f, "Unpack"),
            OpCode::NewArray0 => write!(f, "NewArray0"),
            OpCode::NewArray => write!(f, "NewArray"),
            OpCode::NewArrayT => write!(f, "NewArrayT"),
            OpCode::NewStruct0 => write!(f, "NewStruct0"),
            OpCode::NewStruct => write!(f, "NewStruct"),
            OpCode::NewMap => write!(f, "NewMap"),
            OpCode::Size => write!(f, "Size"),
            OpCode::HasKey => write!(f, "HasKey"),
            OpCode::Keys => write!(f, "Keys"),
            OpCode::Values => write!(f, "Values"),
            OpCode::PickItem => write!(f, "PickItem"),
            OpCode::Append => write!(f, "Append"),
            OpCode::SetItem => write!(f, "SetItem"),
            OpCode::ReverseItems => write!(f, "ReverseItems"),
            OpCode::Remove => write!(f, "Remove"),
            OpCode::ClearItems => write!(f, "ClearItems"),
            OpCode::PopItem => write!(f, "PopItem"),
            OpCode::IsNull => write!(f, "IsNull"),
            OpCode::IsType => write!(f, "IsType"),
            OpCode::Convert => write!(f, "Convert"),
            OpCode::AbortMsg => write!(f, "AbortMsg"),
            OpCode::AssertMsg => write!(f, "AssertMsg"),
        }
    }
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
    pub fn opcode(self) -> u8 {
        self as u8
    }

    pub fn to_string(self) -> String {
        format!("{:02X}", self as u8)
    }

    pub fn operand_size(self) -> Option<OperandSize> {
        match self {
            Self::PushInt8 |
            Self::Jmp |
            Self::JmpIf |
            Self::JmpIfNot |
            Self::JmpEq |
            Self::JmpNe |
            Self::JmpGt |
            Self::JmpGe |
            Self::JmpLt |
            Self::JmpLe |
            Self::Call |
            Self::EndTry |
            Self::InitSSLot |
            Self::LdSFLd |
            Self::StSFLd |
            Self::LdLoc |
            Self::StLoc |
            Self::LdArg |
            Self::StArg |
            Self::NewArrayT |
            Self::IsType |
            Self::Convert => Some(OperandSize::with_size(1)),

            Self::PushInt16 |
            Self::CallT |
            Self::Try |
            Self::InitSlot => Some(OperandSize::with_size(2)),

            Self::PushInt32 |
            Self::PushA |
            Self::JmpL |
            Self::JmpIfL |
            Self::JmpIfNotL |
            Self::JmpEqL |
            Self::JmpNeL |
            Self::JmpGtL |
            Self::JmpGeL |
            Self::JmpLtL |
            Self::JmpLeL |
            Self::CallL |
            Self::EndTryL |
            Self::SysCall => Some(OperandSize::with_size(4)),

            Self::PushInt64 |
            Self::TryL => Some(OperandSize::with_size(8)),

            Self::PushInt128 => Some(OperandSize::with_size(16)),

            Self::PushInt256 => Some(OperandSize::with_size(32)),

            Self::PushData1 => Some(OperandSize::with_prefix_size(1)),
            Self::PushData2 => Some(OperandSize::with_prefix_size(2)),
            Self::PushData4 => Some(OperandSize::with_prefix_size(4)),

            _ => None
        }
    }


    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::PushInt8),
            0x01 => Some(Self::PushInt16),
            0x02 => Some(Self::PushInt32),
            0x03 => Some(Self::PushInt64),
            0x04 => Some(Self::PushInt128),
            0x05 => Some(Self::PushInt256),

            0x0A => Some(Self::PushA),
            0x0B => Some(Self::PushNull),
            0x0C => Some(Self::PushData1),
            0x0D => Some(Self::PushData2),
            0x0E => Some(Self::PushData4),
            0x0F => Some(Self::PushM1),

            0x10 => Some(Self::Push0),
            0x11 => Some(Self::Push1),
            0x12 => Some(Self::Push2),
            0x13 => Some(Self::Push3),
            0x14 => Some(Self::Push4),
            0x15 => Some(Self::Push5),
            0x16 => Some(Self::Push6),
            0x17 => Some(Self::Push7),
            0x18 => Some(Self::Push8),
            0x19 => Some(Self::Push9),
            0x1A => Some(Self::Push10),
            0x1B => Some(Self::Push11),
            0x1C => Some(Self::Push12),
            0x1D => Some(Self::Push13),
            0x1E => Some(Self::Push14),
            0x1F => Some(Self::Push15),
            0x20 => Some(Self::Push16),

            0x21 => Some(Self::Nop),
            0x22 => Some(Self::Jmp),
            0x23 => Some(Self::JmpL),

            0x24 => Some(Self::JmpIf),
            0x25 => Some(Self::JmpIfL),
            0x26 => Some(Self::JmpIfNot),
            0x27 => Some(Self::JmpIfNotL),
            0x28 => Some(Self::JmpEq),
            0x29 => Some(Self::JmpEqL),
            0x2A => Some(Self::JmpNe),
            0x2B => Some(Self::JmpNeL),
            0x2C => Some(Self::JmpGt),
            0x2D => Some(Self::JmpGtL),
            0x2E => Some(Self::JmpGe),
            0x2F => Some(Self::JmpGeL),
            0x30 => Some(Self::JmpLt),
            0x31 => Some(Self::JmpLtL),
            0x32 => Some(Self::JmpLe),
            0x33 => Some(Self::JmpLeL),
            0x34 => Some(Self::Call),
            0x35 => Some(Self::CallL),
            0x36 => Some(Self::CallA),
            0x37 => Some(Self::CallT),
            0x38 => Some(Self::Abort),
            0x39 => Some(Self::Assert),
            0x3A => Some(Self::Throw),
            0x3B => Some(Self::Try),
            0x3C => Some(Self::TryL),
            0x3D => Some(Self::EndTry),
            0x3E => Some(Self::EndTryL),
            0x3F => Some(Self::EndFinally),
            0x40 => Some(Self::Ret),
            0x41 => Some(Self::Syscall),

            0x43 => Some(Self::Depth),
            0x45 => Some(Self::Drop),
            0x46 => Some(Self::Nip),

            0x48 => Some(Self::Xdrop),
            0x49 => Some(Self::Clear),
            0x4A => Some(Self::Dup),
            0x4B => Some(Self::Over),
            0x4C => Some(Self::Pick),
            0x4D => Some(Self::Tuck),
            0x4E => Some(Self::Swap),
            0x50 => Some(Self::Rot),
            0x51 => Some(Self::Roll),
            0x52 => Some(Self::Reverse3),
            0x53 => Some(Self::Reverse4),
            0x54 => Some(Self::ReverseN),
            0x55 => Some(Self::ReverseN),

            // Slot
            0x56 => Some(Self::InitSSLot),
            0x57 => Some(Self::InitSlot),
            0x58 => Some(Self::LdSFLd0),
            0x59 => Some(Self::LdSFLd1),
            0x5A => Some(Self::LdSFLd2),
            0x5B => Some(Self::LdSFLd3),
            0x5C => Some(Self::LdSFLd4),
            0x5D => Some(Self::LdSFLd5),
            0x5E => Some(Self::LdSFLd6),
            0x5F => Some(Self::LdSFLd),

            0x60 => Some(Self::StSFLd0),
            0x61 => Some(Self::StSFLd1),
            0x62 => Some(Self::StSFLd2),
            0x63 => Some(Self::StSFLd3),
            0x64 => Some(Self::StSFLd4),
            0x65 => Some(Self::StSFLd5),
            0x66 => Some(Self::StSFLd6),
            0x67 => Some(Self::StSFLd),

            0x68 => Some(Self::LdLoc0),
            0x69 => Some(Self::LdLoc1),
            0x6A => Some(Self::LdLoc2),
            0x6B => Some(Self::LdLoc3),
            0x6C => Some(Self::LdLoc4),
            0x6D => Some(Self::LdLoc5),
            0x6E => Some(Self::LdLoc6),
            0x6F => Some(Self::LdLoc),

            0x70 => Some(Self::StLoc0),
            0x71 => Some(Self::StLoc1),
            0x72 => Some(Self::StLoc2),
            0x73 => Some(Self::StLoc3),
            0x74 => Some(Self::StLoc4),
            0x75 => Some(Self::StLoc5),
            0x76 => Some(Self::StLoc6),
            0x77 => Some(Self::StLoc),

            0x78 => Some(Self::LdArg0),
            0x79 => Some(Self::LdArg1),
            0x7A => Some(Self::LdArg2),
            0x7B => Some(Self::LdArg3),
            0x7C => Some(Self::LdArg4),
            0x7D => Some(Self::LdArg5),
            0x7E => Some(Self::LdArg6),
            0x7F => Some(Self::LdArg),

            0x80 => Some(Self::StArg0),
            0x81 => Some(Self::StArg1),
            0x82 => Some(Self::StArg2),
            0x83 => Some(Self::StArg3),
            0x84 => Some(Self::StArg4),
            0x85 => Some(Self::StArg5),
            0x86 => Some(Self::StArg6),
            0x87 => Some(Self::StArg),

            // Splice
            0x88 => Some(Self::NewBuffer),
            0x89 => Some(Self::MemCpy),
            0x8B => Some(Self::Cat),
            0x8C => Some(Self::Substr),
            0x8D => Some(Self::Left),
            0x8E => Some(Self::Right),

            // Bitwise Logic
            0x90 => Some(Self::Invert),
            0x91 => Some(Self::And),
            0x92 => Some(Self::Or),
            0x93 => Some(Self::Xor),
            0x97 => Some(Self::Equal),
            0x98 => Some(Self::NotEqual),

            // Arithmetic
            0x99 => Some(Self::Sign),
            0x9A => Some(Self::Abs),
            0x9B => Some(Self::Negate),
            0x9C => Some(Self::Inc),
            0x9D => Some(Self::Dec),
            0x9E => Some(Self::Add),
            0x9F => Some(Self::Sub),
            0xA0 => Some(Self::Mul),
            0xA1 => Some(Self::Div),
            0xA2 => Some(Self::Mod),
            0xA3 => Some(Self::Pow),
            0xA4 => Some(Self::Sqrt),
            0xA5 => Some(Self::ModMul),
            0xA6 => Some(Self::ModPow),
            0xA8 => Some(Self::Shl),
            0xA9 => Some(Self::Shr),
            0xAA => Some(Self::Not),
            0xAB => Some(Self::BoolAnd),
            0xAC => Some(Self::BoolOr),
            0xB1 => Some(Self::Nz),
            0xB3 => Some(Self::NumEqual),
            0xB4 => Some(Self::NumNotEqual),
            0xB5 => Some(Self::Lt),
            0xB6 => Some(Self::Le),
            0xB7 => Some(Self::Gt),
            0xB8 => Some(Self::Ge),
            0xB9 => Some(Self::Min),
            0xBA => Some(Self::Max),

            // Compound-Type
            0xBE => Some(Self::PackMap),
            0xBF => Some(Self::PackStruct),
            0xC0 => Some(Self::Pack),
            0xC1 => Some(Self::Unpack),
            0xC2 => Some(Self::NewArray0),
            0xC3 => Some(Self::NewArray),
            0xC4 => Some(Self::NewArrayT),
            0xC5 => Some(Self::NewStruct0),
            0xC6 => Some(Self::NewStruct),
            0xC8 => Some(Self::NewMap),
            0xCA => Some(Self::Size),
            0xCB => Some(Self::HasKey),
            0xCC => Some(Self::Keys),
            0xCD => Some(Self::Values),
            0xCE => Some(Self::PickItem),
            0xCF => Some(Self::Append),
            0xD0 => Some(Self::SetItem),
            0xD1 => Some(Self::ReverseItems),
            0xD2 => Some(Self::Remove),
            0xD3 => Some(Self::ClearItems),

            // Types
            0xD8 => Some(Self::IsNull),
            0xD9 => Some(Self::IsType),
            0xDB => Some(Self::Convert),

            0xE0 => Some(Self::AbortMsg),
            OxE1 =>Some(Self::AssertMsg),
            _ => None
        }
    }

}

#[derive(Clone, Copy, Debug)]
pub struct OperandSize {
    prefix_size: u8,
    size: u8,
}

impl OperandSize {
    pub fn with_size(size: u8) -> Self {
        Self { prefix_size: 0, size }
    }

    pub fn with_prefix_size(prefix_size: u8) -> Self {
        Self { prefix_size, size: 0 }
    }
}