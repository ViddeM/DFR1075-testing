// Wrapper type that just implements CryptoRng for esp_hal's RngCore
// TODO: find out if there's a more cryptographically secure way of doing this.
pub struct RngWrapper {
    rng: esp_hal::rng::Rng,
}

impl From<esp_hal::rng::Rng> for RngWrapper {
    fn from(rng: esp_hal::rng::Rng) -> Self {
        RngWrapper { rng }
    }
}

impl rand_core::RngCore for RngWrapper {
    fn next_u32(&mut self) -> u32 {
        self.rng.random()
    }

    fn next_u64(&mut self) -> u64 {
        (self.rng.random() as u64) << 32 | self.rng.random() as u64
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for b in dest {
            *b = (self.rng.random() & 0xff) as u8;
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl rand_core::CryptoRng for RngWrapper {}
