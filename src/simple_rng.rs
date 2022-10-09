#![allow(dead_code)]

/// My Own Simple RNG

use std::ops::Range;
use std::mem::transmute;


pub struct SimpleRng {
    seed: u32
}

impl SimpleRng {
    pub const fn new(seed: u32) -> Self {
        SimpleRng { seed }
    }

    fn update_seed(&mut self) {
        self.seed = self.seed
            .overflowing_mul(5).0
            .overflowing_add(1).0;
    }

    pub fn random_float_from_0_to_1(&mut self) -> f32 {
        self.update_seed();
        //                      sign  exponent   fraction
        const MASK_AND: u32 = 0b___0__0000_0000__111_1111_1111_1111_1111_1111;

        // const MASK_OR : u32 = 0b___0__0000_0000__000_0000_0000_0000_0000_0000; // 7
        // const MASK_OR : u32 = 0b___0__0100_0000__000_0000_0000_0000_0000_0000; // 6
        // const MASK_OR : u32 = 0b___0__0110_0000__000_0000_0000_0000_0000_0000; // 5
        // const MASK_OR : u32 = 0b___0__0111_0000__000_0000_0000_0000_0000_0000; // 4
        // const MASK_OR : u32 = 0b___0__0111_1000__000_0000_0000_0000_0000_0000; // 3
        // const MASK_OR : u32 = 0b___0__0111_1100__000_0000_0000_0000_0000_0000; // 2
        // const MASK_OR : u32 = 0b___0__0111_1110__000_0000_0000_0000_0000_0000; // 1
        const MASK_OR : u32 = 0b___0__0111_1111__000_0000_0000_0000_0000_0000; // 0

        let res: u32 = self.seed;
        let res: u32 = res & MASK_AND;
        let res: u32 = res | MASK_OR;
        let res: f32 = unsafe { transmute::<u32, f32>(res) };

        // let res: f32 = res * 170141183460469231731687303715884105728.0 - 1.0; // 7
        // const MASK_RM_SIGN: u32 = 0b___1__0000_0000__000_0000_0000_0000_0000_0000; // mask to remove sign from f32
        // let res: f32 = unsafe { transmute::<u32, f32>(transmute::<f32, u32>(res) & MASK_RM_SIGN) };
        // let res: f32 = res * 9223372036854775808.0 - 1.0; // 6
        // let res: f32 = res * 2147483648.0 - 1.0; // 5
        // let res: f32 = res * 32768.0 - 1.0; // 4
        // let res: f32 = res * 128.0 - 1.0; // 3
        // let res: f32 = res * 8.0 - 1.0; // 2
        // let res: f32 = res * 2.0 - 1.0; // 1
        let res: f32 = res - 1.0; // 0   ~7% faster :rocket:
        res
    }

    pub fn gen_bool(&mut self, p: f32) -> bool {
        self.random_float_from_0_to_1() < p
    }

    pub fn gen_range(&mut self, range: Range<f32>) -> f32 {
        range.start + (range.end - range.start) * self.random_float_from_0_to_1()
    }

    pub fn gen_range_u32(&mut self, range: Range<u32>) -> u32 {
        range.start + ((range.end - range.start) as f32 * self.random_float_from_0_to_1()) as u32
    }

    pub fn gen_range_usize(&mut self, range: Range<usize>) -> usize {
        range.start + ((range.end - range.start) as f32 * self.random_float_from_0_to_1()) as usize
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_seed() {
        const SEED: u32 = 0;
        let mut rng: SimpleRng = SimpleRng::new(SEED);
        let mut numbers_generated: u64 = 0;
        loop {
            let prev_seed = rng.seed;
            rng.update_seed();
            numbers_generated += 1;
            if rng.seed == prev_seed {
                panic!("seed was same!");
            }
            else if rng.seed == SEED {
                break;
            }
        }
        assert_eq!(
            u32::MAX as u64 + 1, // +1 bcof 0 is also number in u32
            numbers_generated as u64
        );
    }


    #[test]
    fn random_float_from_0_to_1() {
        const SEED: u32 = 0;
        let mut rng: SimpleRng = SimpleRng::new(SEED);
        for _ in u32::MIN..u32::MAX {
            let x: f32 = rng.random_float_from_0_to_1();
            assert!(0.0_f32 <= x && x <= 1.0_f32);
        }
    }


    #[test]
    fn gen_range() {
        const SEED: u32 = 0;
        let mut rng: SimpleRng = SimpleRng::new(SEED);
        const MIN: f32 = 7.2;
        const MAX: f32 = 42.145;
        for _ in u32::MIN..u32::MAX {
            let random_float: f32 = rng.gen_range(MIN..MAX);
            assert!(MIN <= random_float && random_float <= MAX);
        }
    }


    #[test]
    fn gen_range_u32() {
        const SEED: u32 = 0;
        let mut rng: SimpleRng = SimpleRng::new(SEED);
        const MIN: u32 = 7;
        const MAX: u32 = 42;
        for _ in u32::MIN..u32::MAX {
            let random_u32: u32 = rng.gen_range_u32(MIN..MAX);
            assert!(MIN <= random_u32 && random_u32 <= MAX);
        }
    }


    #[test]
    fn gen_range_usize() {
        const SEED: u32 = 0;
        let mut rng: SimpleRng = SimpleRng::new(SEED);
        const MIN: usize = 7;
        const MAX: usize = 42;
        for _ in u32::MIN..u32::MAX {
            let random_usize: usize = rng.gen_range_usize(MIN..MAX);
            assert!(MIN <= random_usize && random_usize <= MAX);
        }
    }


    #[test]
    fn random_float_from_0_to_1_statistics() {
        const SEED: u32 = 0;
        const BUCKETS: usize = 2_usize.pow(18); // it will fail if pow>23, cause this rng have only 23 dimensions of freedom
        let mut stats: Vec<u32> = vec![0; BUCKETS];
        let mut rng: SimpleRng = SimpleRng::new(SEED);
        for _ in u32::MIN..=u32::MAX {
            let x: f32 = rng.random_float_from_0_to_1();
            let is_in_range: bool = 0.0 <= x && x <= 1.0;
            if !is_in_range {
                panic!("random_float_from_0_to_1 NOT IN RANGE 0.0 .. 1.0 : {x}");
            }
            stats[(x * BUCKETS as f32) as usize] += 1;
        }
        // println!("{{");
        // for b in 0..BUCKETS {
        //     println!("    {b}: {v}", v=stats[b]);
        // }
        // println!("}}");
        // panic!("OK. Printed results.");
        for i in 0..stats.len()-1 {
            assert_eq!(stats[i], stats[i+1]);
        }
    }

}



#[cfg(test)]
mod benches {
    use test::{Bencher, black_box};

    #[bench]
    fn bench_random_float_from_0_to_1(b: &mut Bencher) {
        use super::SimpleRng;
        b.iter(|| {
            const SEED: u32 = 0;
            let mut rng: SimpleRng = SimpleRng::new(SEED);
            for _ in u32::MIN..u32::MAX {
                let x: f32 = rng.random_float_from_0_to_1();
                // let x: f32 = rng.gen_range(0.0_f32..1.0_f32);
                black_box(x);
            }
        });
    }

    #[ignore]
    #[bench]
    fn bench_rand_lib_float_from_0_to_1(b: &mut Bencher) {
        // use rand::{Rng, prelude::ThreadRng, thread_rng};
        b.iter(|| {
            panic!("plz enable `rand`");
            // const SEED: u32 = 0;
            // let mut rng: ThreadRng = thread_rng();
            // for _ in u32::MIN..u32::MAX {
            //     let x: f32 = rng.gen_range(0.0_f32..1.0_f32);
            //     black_box(x);
            // }
        });
    }

}

