#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RecordType {
    A = 1,
    CNAME = 5,
    TXT = 16,
    AAAA = 28,
}

impl RecordType {

    pub fn to_string(&self) -> &str {
        match self {
            RecordType::A => "A",
            RecordType::CNAME => "CNAME",
            RecordType::TXT => "TXT",
            RecordType::AAAA => "AAAA",
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(RecordType::A),
            5 => Some(RecordType::CNAME),
            16 => Some(RecordType::TXT),
            28 => Some(RecordType::AAAA),
            _ => None,
        }
    }

    pub fn byte_repr(self) -> u8 {
        self as u8
    }

}