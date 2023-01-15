pub mod xoro {
    use std::ops::Range;

    #[derive(Clone)]
    pub struct Xoro {
        pub seed_range: Range<i64>,
    }

    impl Xoro {
        /// Creates new Xoroshiro128++ number generater with a 64-bit seed
        pub fn new(seed: i64) -> Xoro {
            let l = seed ^ 0x6A09E667F3BCC909;
            let m = l.wrapping_sub(7046029254386353131);
            Xoro {
                seed_range: Xoro::next_split_mix_i64(l)..Xoro::next_split_mix_i64(m),
            }
        }

        /// Ngl idk what this really does
        fn next_split_mix_i64(seed: i64) -> i64 {
            let seed = (seed ^ (seed as u64 >> 30) as i64).wrapping_mul(-4658895280553007687);
            let seed = (seed ^ (seed as u64 >> 27) as i64).wrapping_mul(-7723592293110705685);
            seed ^ (seed as u64 >> 31) as i64
        }

        pub fn copy_seed_to(&self, random: &mut Xoro) {
            random.seed_range.start = self.seed_range.start;
            random.seed_range.end = self.seed_range.end;
        }

        /// Set the seed
        pub fn set_seed(&mut self, seed: i64) {
            let l = seed ^ 0x6A09E667F3BCC909;
            let m = l.wrapping_sub(7046029254386353131);
            self.seed_range = Xoro::next_split_mix_i64(l)..Xoro::next_split_mix_i64(m);
        }

        /// Gets the next 64-bit integer
        pub fn next_i64(&mut self) -> i64 {
            let l = self.seed_range.start;
            let mut m = self.seed_range.end;
            let n = l.wrapping_add(m).rotate_left(17).wrapping_add(l);
            self.seed_range.start = l.rotate_left(49)
                ^ { m ^= l; m }
                ^ m << 21;
            self.seed_range.end = m.rotate_left(28);
            n
        }

        /// Skip the next `count` calls to `next_i64()`
        pub fn skip(&mut self, count: i32) {
            for _ in 0..count {
                self.next_i64();
            }
        }
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
            let mut x = Xoro::new(111);
            assert_eq!(17056846968231911, x.next_i64());
            assert_eq!(3500609826288335747, x.next_i64());
            assert_eq!(7979107359202306130, x.next_i64());
        }
    }
}

pub mod chunk_random {
    use std::num::Wrapping;
    use std::rc::Rc;
    use std::cell::RefCell;
    use super::xoro::Xoro;

    pub struct ChunkRandom {
        base_random: Rc<RefCell<Xoro>>,
    }

    impl ChunkRandom {
        pub fn new(base_random: Rc<RefCell<Xoro>>) -> ChunkRandom {
            // let base_random = Xoro::new(1);
            ChunkRandom { base_random }
        }

        pub fn next(&mut self, count: i32) -> i32 {
            ((*self.base_random.borrow_mut()).next_i64() as u64 >> 64 - count) as i32
        }

        pub fn next_int(&mut self, bound: u32) -> i32 {
            let bound = bound as i32;
            assert!(bound > 0);

            // i.e., bound is a power of 2
            if (bound & -bound) == bound {
                return (((bound as i64) * (self.next(31) as i64)) >> 31) as i32;
            }

            let mut bits;
            let mut val;

            loop {
                bits = self.next(31);
                val = bits % bound;
                
                if bits - val + (bound - 1) >= 0
                {
                    return val;
                }
            }
        }

        pub fn next_i64(&mut self) -> i64 {
            ((self.next(32) as i64) << 32) + self.next(32) as i64
        }

        pub fn set_seed(&mut self, seed: i64) {
            (*self.base_random.borrow_mut()).set_seed(seed);
        }

        pub fn set_population_seed(&mut self, world_seed: i64, block_x: i32, block_z: i32) -> i64 {
            self.set_seed(world_seed);
            let l = Wrapping(self.next_i64() | 1);
            let m = Wrapping(self.next_i64() | 1);
            let block_x = Wrapping(block_x as i64);
            let block_z = Wrapping(block_z as i64);
            let n = (block_x * l + block_z * m ^ Wrapping(world_seed)).0;
            self.set_seed(n);
            n
        }
    }

    #[cfg(test)]
    mod tests {
        use std::rc::Rc;

        use super::*;

        #[test]
        fn it_works() {
            let mut x = Rc::new(RefCell::new(Xoro::new(1)));
            let mut c = ChunkRandom::new(Rc::clone(&x));
            assert_eq!(61863, c.next(16));
            assert_eq!(22920, c.next(16));
            assert_eq!(59063, c.next(16));
        }

        #[test]
        fn next_int_bound() {
            let mut x = Rc::new(RefCell::new(Xoro::new(1)));
            let mut c = ChunkRandom::new(Rc::clone(&x));
            assert_eq!(15, c.next_int(16));
            assert_eq!(5, c.next_int(16));
            assert_eq!(14, c.next_int(16));
        }
    }
}

/// Referenced from:
/// https://github.com/Xydez/javarandom-rs/blob/master/src/java_random.rs
/// (licensed under the Unlicense license, ty)
pub mod java_random {
    use std::num::Wrapping;

    pub struct JavaRandom {
        seed: Wrapping<i64>,
    }

    impl Default for JavaRandom {
        fn default() -> Self {
            JavaRandom::with_seed(0)
        }
    }

    impl JavaRandom {
        pub fn with_seed(seed: i64) -> JavaRandom {
            let seed = Wrapping(seed);

            JavaRandom {
                seed: (seed ^ Wrapping(0x5DEECE66Di64)) & Wrapping((1i64 << 48) - 1),
            }
        }

        fn next(&mut self, bits: i32) -> i32 {
            self.seed = (self.seed * Wrapping(0x5DEECE66Di64) + Wrapping(0xBi64))
                & Wrapping((1i64 << 48) - 1);

            (self.seed.0 as u64 >> (48 - bits)) as i32
        }

        pub fn next_i64(&mut self) -> i64 {
            ((self.next(32) as i64) << 32) + self.next(32) as i64
        }
    }
}
