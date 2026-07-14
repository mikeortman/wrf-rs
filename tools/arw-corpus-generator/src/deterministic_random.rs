pub(crate) struct DeterministicRandom {
    state: u64,
}

impl DeterministicRandom {
    pub(crate) const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    pub(crate) fn usize_inclusive(&mut self, minimum: usize, maximum: usize) -> usize {
        debug_assert!(minimum <= maximum);
        minimum + self.next_u64() as usize % (maximum - minimum + 1)
    }

    pub(crate) fn i32_inclusive(&mut self, minimum: i32, maximum: i32) -> i32 {
        debug_assert!(minimum <= maximum);
        minimum + (self.next_u64() % (i64::from(maximum) - i64::from(minimum) + 1) as u64) as i32
    }

    pub(crate) fn moderate_f32_bits(&mut self) -> u32 {
        (self.i32_inclusive(-32_000, 32_000) as f32 * 0.031_25).to_bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix_sequence_is_stable() {
        let mut random = DeterministicRandom::new(1);
        assert_eq!(random.next_u64(), 0x910A_2DEC_8902_5CC1);
        assert_eq!(random.next_u64(), 0xBEEB_8DA1_658E_EC67);
        assert_eq!(random.next_u64(), 0xF893_A2EE_FB32_51E9);
    }
}
