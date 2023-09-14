// Numeric.rs


use std::ops::Add;

pub trait Padded<T> {
    fn to_bytes_padded(&self, length: usize) -> Vec<u8>;
}

impl<T> Padded<T> for T
where
    T:  Add<Output = T>, {
    fn to_bytes_padded(&self, length: usize) -> Vec<u8> {
        let mut bytes = self.to_be_bytes().to_vec();
        bytes.resize(length, 0);
        Vec::from(bytes)
    }
}