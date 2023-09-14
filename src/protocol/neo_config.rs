use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;

pub const DEFAULT_BLOCK_TIME: u64 = 15_000;
pub const DEFAULT_ADDRESS_VERSION: u8 = 0x35;
pub const MAX_VALID_UNTIL_BLOCK_INCREMENT_BASE: u64 = 86_400_000;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NeoConfig {
    pub network_magic: Option<u32>,
    pub block_interval: u64,
    pub max_valid_until_block_increment: u64,
    pub polling_interval: u64,
    executor: Handle,
    pub allows_transmission_on_fault: bool,
    pub nns_resolver: [u8; 20],
}

impl Default for NeoConfig {
    fn default() -> Self {
        NeoConfig {
            network_magic: None,
            block_interval: DEFAULT_BLOCK_TIME,
            max_valid_until_block_increment: MAX_VALID_UNTIL_BLOCK_INCREMENT_BASE / DEFAULT_BLOCK_TIME,
            polling_interval: DEFAULT_BLOCK_TIME,
            executor: Handle::current(),
            allows_transmission_on_fault: false,
            nns_resolver: [0x50, 0xac, 0x1c, 0x37, 0x69, 0x0c, 0xc2, 0xc5, 0x8f, 0xc5, 0x94, 0x47, 0x28, 0x33, 0xcf, 0x57, 0x50, 0x5d, 0x5f, 0x46],
        }
    }
}

impl NeoConfig {
    // constructor
    pub fn new() -> Self {
        Default::default()
    }

    // setters
    pub fn set_polling_interval(&mut self, interval: u64) {
        self.polling_interval = interval;
    }

    pub fn set_executor(&mut self, executor: tokio::runtime::Handle) {
        self.executor = executor;
    }

    pub fn set_network_magic(&mut self, magic: u32) -> Result<(), &'static str> {
        if &magic > 0xFFFFFFFF {
            return Err("Network magic must fit in 32 bits");
        }

        self.network_magic = Some(magic);
        Ok(())
    }

    // other methods
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Counter {
    count: Arc<Mutex<u32>>,
}

impl Counter {
    pub fn new() -> Self {
        Counter {
            count: Arc::new(Mutex::new(1))
        }
    }

    pub fn get_and_increment(&self) -> u32 {
        let mut count = self.count.lock().unwrap();
        let v: u32 = *count;
        *count += 1;
        v
    }
}