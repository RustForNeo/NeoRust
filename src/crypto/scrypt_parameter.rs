use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ScryptParams {
    #[serde(rename="n")]
    pub n: u32,

    #[serde(rename="r")]
    pub r: u32,

    #[serde(rename="p")]
    pub p: u32
}

impl ScryptParams {

    pub const DEFAULT: Self = Self {
        n: 1 << 14,
        r: 8,
        p: 1
    };

    pub fn new(n: u32, r: u32, p: u32) -> Self {
        Self { n, r, p }
    }
}