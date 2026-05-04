/// Splitmix64-based PRNG. Deterministic, no system entropy.
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn next_usize(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound
    }

    pub fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let mut a = SeededRng::new(42);
        let mut b = SeededRng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_differ() {
        let mut a = SeededRng::new(1);
        let mut b = SeededRng::new(2);
        let vals_a: Vec<u64> = (0..10).map(|_| a.next_u64()).collect();
        let vals_b: Vec<u64> = (0..10).map(|_| b.next_u64()).collect();
        assert_ne!(vals_a, vals_b);
    }

    #[test]
    fn next_usize_within_bound() {
        let mut rng = SeededRng::new(0);
        for _ in 0..1000 {
            assert!(rng.next_usize(10) < 10);
        }
    }
}
