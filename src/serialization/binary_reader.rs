use bitcoin::consensus::ReadExt;
use bitcoin::Script;
use serde::Deserialize;
use crate::neo_error::NeoRustError;
use crate::types::ECPoint;

pub struct BinaryReader<'a> {
    data: &'a [u8],
    position: usize,
    marker: usize,
}

impl<'a> BinaryReader<'a> {

    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            position: 0,
            marker: 0,
        }
    }

    pub fn read_bool(&mut self) -> bool {
        let val = self.data[self.position] == 1;
        self.position += 1;
        val
    }

    pub fn read_u8(&mut self) -> u8 {
        let val = self.data[self.position];
        self.position += 1;
        val
    }

    // Other primitive reader methods

    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], NeoRustError> {
        let start = self.position;
        self.position += count;
        self.data.get(start..self.position)
            .ok_or_else(|| NeoRustError::IndexOutOfBounds("Out of bounds".to_string()))
    }

    pub fn read_var_bytes(&mut self) -> Result<&'a [u8], NeoRustError> {
        let len = self.read_var_int()? as usize;
        self.read_bytes(len)
    }

    pub fn read_var_int(&mut self) -> Result<i64, NeoRustError> {
        let first = self.read_u8();
        match first {
            0xfd => Ok(self.read_u16()? as i64),
            0xfe => Ok(self.read_u32()? as i64),
            0xff => Ok(self.read_u64()? as i64),
            _ => Ok(first as i64)
        }
    }

    // Serialization helper methods

    pub fn read_serializable<T: Deserialize>(&mut self) -> Result<T, NeoRustError> {
        T::deserialize(self)
    }

    pub fn read_serializable_list<T: Deserialize>(&mut self) -> Result<Vec<T>, NeoRustError> {
        let len = self.read_var_int()?;
        let mut list = Vec::with_capacity(len as usize);
        for _ in 0..len {
            list.push(self.read_serializable()?);
        }
        Ok(list)
    }

    // Other methods like `mark`, `reset`, etc.

    pub fn mark(&mut self) {
        self.marker = self.position;
    }

    pub fn reset(&mut self) {
        self.position = self.marker;
    }

    pub fn read_script(&mut self) -> Result<Script, NeoRustError> {
        let len = self.read_var_int()?;
        let bytes = self.read_bytes(len as usize)?;
        Ok(Script::from_bytes(bytes))
    }

    pub fn read_ec_point(&mut self) -> Result<ECPoint, NeoRustError> {
        let byte = self.read_u8();
        let bytes = match byte {
            0x00 => [0].to_vec(),
            0x02 | 0x03 => {
                let mut bytes = Vec::with_capacity(33);
                bytes.push(byte);
                bytes.extend_from_slice(&self.read_bytes(32)?);
                bytes
            },
            0x04 => {
                let mut bytes = Vec::with_capacity(65);
                bytes.push(byte);
                bytes.extend_from_slice(&self.read_bytes(64)?);
                bytes
            },
            _ => return Err(NeoRustError::InvalidData("Invalid EC point encoding".to_string()))
        };

        ECPoint::decode(&SECP256R1, &bytes)
            .map_err(|e| NeoRustError::InvalidData(e.to_string()))
    }
}
