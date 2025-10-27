/// Deterministic pseudo-random generator for reproducible puzzles.
#[derive(Clone)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: Option<u64>, salt_a: u64, salt_b: u64) -> Self {
        let mut state = seed.unwrap_or(0x9e37_79b9_7f4a_7c15 ^ salt_a.wrapping_shl(16) ^ salt_b);
        if state == 0 {
            state = 0xfeed_c0de_dead_beef;
        }
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        const A: u64 = 6364136223846793005;
        const C: u64 = 1;
        self.state = self.state.wrapping_mul(A).wrapping_add(C);
        self.state
    }

    pub fn gen_range_inclusive(&mut self, min: u8, max: u8) -> u8 {
        let span = (max - min + 1) as u64;
        let value = self.next_u64() % span;
        min + value as u8
    }

    pub fn gen_range_usize(&mut self, min: usize, max: usize) -> usize {
        let span = (max - min) as u64;
        if span == 0 {
            return min;
        }
        (self.next_u64() % (span + 1)) as usize + min
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = (self.next_u64() % (i as u64 + 1)) as usize;
            slice.swap(i, j);
        }
    }
}
