// Author: Dylan Jones
// Date:   2025-05-15

use rand::RngCore;

pub struct RandomIdGenerator {
    is_28_bit: bool,
}

impl RandomIdGenerator {
    pub fn new(is_28_bit: bool) -> Self {
        Self { is_28_bit }
    }
}

impl Iterator for RandomIdGenerator {
    type Item = anyhow::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rng = rand::rng();
        let mut buf = [0u8; 4];
        let max_retry = 1_000_000;
        for _ in 0..max_retry {
            rng.fill_bytes(&mut buf);
            let mut id = ((buf[0] as u32) << 24)
                + ((buf[1] as u32) << 16)
                + ((buf[2] as u32) << 8)
                + (buf[3] as u32);
            if self.is_28_bit {
                id >>= 4;
            }
            if id < 100 {
                continue;
            }
            return Some(Ok(format!("{}", id)));
        }
        Some(Err(anyhow::anyhow!(
            "Failed to generate a unique ID after {} attempts",
            max_retry
        )))
    }
}
