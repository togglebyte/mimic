use std::time::{SystemTime, UNIX_EPOCH};

pub struct Random {
    state: u64,
}

impl Random {
    pub fn new() -> Self {
        let state = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("UNIX_EPOCH is always in the past")
            .as_millis() as u64;
        Self { state }
    }

    pub fn next(&mut self, max: u64) -> u64 {
        self.state ^= self.state >> 13;
        self.state ^= self.state << 5;
        self.state ^= self.state >> 17;
        self.state % max
    }
}
