use std::hash::Hasher;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryWriter {
    data: Vec<u8>,
}

impl BinaryWriter {

    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    // Other primitive write methods

    pub fn write_var_int(&mut self, value: i64) {
        match value {
            0..=0xfd => self.write_u8(value as u8),
            0x10000..=0xffffffff => {
                self.write_u8(0xfd);
                self.write_u16(value as u16);
            },
            _ => {
                self.write_u8(0xff);
                self.write_u64(value as u64);
            }
        }
    }

    pub fn write_var_bytes(&mut self, bytes: &[u8]) {
        self.write_var_int(bytes.len() as i64);
        self.write_bytes(bytes);
    }

    // Serialization helpers

    pub fn write_serializable<S: Serialize>(&mut self, value: &S) {
        value.serialize(self);
    }

    pub fn write_serializable_list<S: Serialize>(&mut self, values: &[S]) {
        self.write_var_int(values.len() as i64);
        for item in values {
            self.write_serializable(item);
        }
    }

    pub fn reset(&mut self) {
        self.data.clear();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

}