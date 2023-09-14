use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PopulatedBlocks {
    pub cache_id: String,
    pub blocks: Vec<i32>,
}

impl PopulatedBlocks {

    pub fn new(cache_id: String, blocks: Vec<i32>) -> Self {
        Self {
            cache_id,
            blocks
        }
    }

}

impl Hash for PopulatedBlocks {

    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cache_id.hash(state);
        self.blocks.hash(state);
    }

}