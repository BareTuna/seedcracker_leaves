use std::num::Wrapping;

/// JavaRandom partially implements some functions found in
/// java.util.Random, just the ones needed for this program.
pub trait JavaRandom {
    fn next(&mut self, count: i32) -> i32;

    /// Returns a random number ranging from 0 to up to `bound` (exclusive)
    /// # Panics
    /// Will panic if `bound == 0`
    fn next_i32_bounded(&mut self, bound: u32) -> i32 {
        let bound = bound as i32;
        assert!(bound > 0);

        // i.e., bound is a power of 2
        if (bound & -bound) == bound {
            return ((bound as i64 * self.next(31) as i64) >> 31) as i32;
        }

        let mut bits;
        let mut val;

        loop {
            bits = self.next(31);
            val = bits % bound;

            if bits - val + (bound - 1) >= 0 {
                return val;
            }
        }
    }

    fn next_i64(&mut self) -> i64 {
        ((self.next(32) as i64) << 32) + self.next(32) as i64
    }

    fn set_seed(&mut self, seed: i64);
}

pub trait ChunkRandom {
    /// Seeds the randomizer to create population features such as decorators
    /// and animals.
    ///
    /// This method takes in the world seed and the negative-most block
    /// coordinates of the chunk. The coordinate pair provided is equivalent to
    /// (chunk_x * 16, chunk_z * 16). The three values are mixed together
    /// through some layers of hashing to produce the population seed.
    ///
    /// This function has been proved to be reversible through some exploitation
    /// of the underlying nextLong() weaknesses. It is also important to
    /// remember that since setSeed() truncates the 16 upper bits of world seed,
    /// only the 48 lowest bits affect the population seed output.
    fn set_population_seed(&mut self, world_seed: i64, block_x: i32, block_z: i32) -> i64;
}

/// Xoroshiro128+ is a fast algorithm for computing pseudo-random numbers.
///
/// It is used in Minecraft in favor over Java's built-in pseudo-random number
/// generator.
#[derive(Clone)]
pub struct Xoro {
    hi_lo: (i64, i64),
}

impl Xoro {
    pub fn new(seed: i64) -> Xoro {
        let l = seed ^ 0x6A09E667F3BCC909;
        let m = l.wrapping_sub(7046029254386353131);
        Xoro {
            hi_lo: (Xoro::next_split_mix_i64(l), Xoro::next_split_mix_i64(m)),
        }
    }

    /// Skip the next `count` calls
    pub fn skip(&mut self, count: i32) {
        for _ in 0..count {
            self.next_xoro_i64();
        }
    }

    fn next_split_mix_i64(seed: i64) -> i64 {
        let seed = (seed ^ (seed as u64 >> 30) as i64).wrapping_mul(-4658895280553007687);
        let seed = (seed ^ (seed as u64 >> 27) as i64).wrapping_mul(-7723592293110705685);
        seed ^ (seed as u64 >> 31) as i64
    }

    fn next_xoro_i64(&mut self) -> i64 {
        let l = self.hi_lo.0;
        let mut m = self.hi_lo.1;
        let n = l.wrapping_add(m).rotate_left(17).wrapping_add(l);
        self.hi_lo.0 = l.rotate_left(49) ^ { m ^= l; m } ^ m << 21;
        self.hi_lo.1 = m.rotate_left(28);
        n
    }
}

impl JavaRandom for Xoro {
    fn next(&mut self, count: i32) -> i32 {
        (self.next_xoro_i64() as u64 >> (64 - count)) as i32
    }

    fn set_seed(&mut self, seed: i64) {
        let l = seed ^ 0x6A09E667F3BCC909;
        let m = l.wrapping_sub(7046029254386353131);
        self.hi_lo = (Xoro::next_split_mix_i64(l), Xoro::next_split_mix_i64(m));
    }
}

impl ChunkRandom for Xoro {
    fn set_population_seed(&mut self, world_seed: i64, block_x: i32, block_z: i32) -> i64 {
        self.set_seed(world_seed);
        let l = Wrapping(self.next_i64() | 1);
        let m = Wrapping(self.next_i64() | 1);
        let block_x = Wrapping(block_x as i64);
        let block_z = Wrapping(block_z as i64);
        let n = ((block_x * l + block_z * m) ^ Wrapping(world_seed)).0;
        self.set_seed(n);
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_xoro_i64_works() {
        let mut x = Xoro::new(111);
        assert_eq!(17056846968231911, x.next_xoro_i64());
        assert_eq!(3500609826288335747, x.next_xoro_i64());
        assert_eq!(7979107359202306130, x.next_xoro_i64());
    }

    #[test]
    fn next_works() {
        let mut x = Xoro::new(1);
        assert_eq!(61863, x.next(16));
        assert_eq!(22920, x.next(16));
        assert_eq!(59063, x.next(16));
    }

    #[test]
    fn next_i32_bounded_works() {
        let mut x = Xoro::new(1);
        assert_eq!(15, x.next_i32_bounded(16));
        assert_eq!(5, x.next_i32_bounded(16));
        assert_eq!(14, x.next_i32_bounded(16));
    }
}
