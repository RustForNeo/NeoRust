use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NeoWitness {
    pub invocation: String,
    pub verification: String,
}

impl NeoWitness {
    pub fn new(invocation: String, verification: String) -> Self {
        Self {
            invocation,
            verification,
        }
    }
}