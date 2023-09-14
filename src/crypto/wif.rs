use bitcoin::base58;
use sha2::{Sha256, Digest};

pub trait Wif {
    fn to_wif(&self) -> String;
    fn from_wif(s: &str) -> Option<Vec<u8>>;
}

impl Wif for [u8] {

    fn to_wif(&self) -> String {
        if self.len() != 32 {
            return String::new();
        }

        let mut extended = vec![0x80];
        extended.extend_from_slice(self);
        extended.push(0x01);

        let hash = Sha256::digest(&Sha256::digest(&extended));
        extended.extend_from_slice(&hash[0..4]);

        base58::encode(extended.as_slice()).into_string()
    }

    fn from_wif(s: &str) -> Option<Vec<u8>> {
        let data = base58::decode(s)
            .unwrap();

        if &data.len() != 38 || &data[0] != 0x80 || &data[33] != 0x01 {
            return None;
        }

        let checksum = &Sha256::digest(&Sha256::digest(&data[0..34]))[0..4];
        if checksum != &data[34..] {
            return None;
        }

        Some(data[1..33].to_vec())
    }

}