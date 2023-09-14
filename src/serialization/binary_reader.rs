use bitcoin::consensus::ReadExt;
use bitcoin::Script;
use serde::Deserialize;
use crate::neo_error::NeoRustError;

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

    pub fn read_u16(&mut self) -> u16 {
        let bytes = self.read_bytes(2);
        u16::from_ne_bytes(bytes.try_into().unwrap())
    }

    pub fn read_i16(&mut self) -> i16 {
        let bytes = self.read_bytes(2);
        i16::from_ne_bytes(bytes.try_into().unwrap())
    }

    pub fn read_u32(&mut self) -> u32 {
        let bytes = self.read_bytes(4);
        u32::from_ne_bytes(bytes.try_into().unwrap())
    }

    pub fn read_i32(&mut self) -> i32 {
        let bytes = self.read_bytes(4);
        i32::from_ne_bytes(bytes.try_into().unwrap())
    }

    pub fn read_u64(&mut self) -> u64 {
        let bytes = self.read_bytes(8);
        u64::from_ne_bytes(bytes.try_into().unwrap())
    }

    pub fn read_i64(&mut self) -> i64 {
        let bytes = self.read_bytes(8);
        i64::from_ne_bytes(bytes.try_into().unwrap())
    }

    pub fn read_u128(&mut self) -> u128 {
        let bytes = self.read_bytes(16);
        u128::from_ne_bytes(bytes.try_into().unwrap())
    }

    pub fn read_i128(&mut self) -> i128 {
        let bytes = self.read_bytes(16);
        i128::from_ne_bytes(bytes.try_into().unwrap())
    }

    // pub fn read_bigint(&mut self) -> BigInt::Sign {
    //     let size = self.read_varint();
    //     let bytes = self.read_bytes(size as usize);
    //     BigInt::Sign::parse_bytes(bytes, BigInt::Sign)
    // }

    pub fn read_encoded_ec_point(&mut self) -> Result<&'a [u8], &'static str> {
        let byte = self.read_byte();
        match byte {
            0x02 | 0x03 => Ok(self.read_bytes(32).unwrap()),
            _ => Err("Invalid encoded EC point"),
        }
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
        Ok(Script::from_bytes(bytes).clone().into())
    }

    pub fn read_ec_point(&mut self) -> Result<ECPoint, NeoRustError> {
        pub fn read_ec_point(&mut self) -> Result<ProjectivePoint, &'static str> {
            let tag = self.read_byte();
            let bytes = match tag {
                0x00 => return Ok(ProjectivePoint::IDENTITY),
                0x02 | 0x03 => self.read_bytes(32),
                0x04 => self.read_bytes(64),
                _ => return Err("Invalid EC point tag")
            };

            let point = EncodedPoint::from_bytes(bytes);
            match ProjectivePoint::from_encoded_point(&point) {
                Some(point) => Ok(point),
                None => Err("Invalid EC point")
            }
        }
    }
}
