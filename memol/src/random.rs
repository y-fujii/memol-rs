// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

pub type Generator = XoroShiro128StarStar;

// Xoroshiro128** PRNG, by David Blackman and Sebastiano Vigna <vigna@acm.org>.
// ref. <http://xoshiro.di.unimi.it/xoroshiro128starstar.c>
pub struct XoroShiro128StarStar {
    s0: cell::Cell<u64>,
    s1: cell::Cell<u64>,
}

impl XoroShiro128StarStar {
    pub fn new() -> Self {
        Self {
            s0: cell::Cell::new(0x243f_6a88_85a3_08d3), // OEIS A062964.
            s1: cell::Cell::new(0x93c4_67e3_7db0_c7a4), // OEIS A170874.
        }
    }

    pub fn next_u64(&self) -> u64 {
        let s0 = self.s0.get();
        let s1 = self.s1.get();
        let t = s0 ^ s1;
        self.s0.set(s0.rotate_left(24) ^ t ^ (t << 16));
        self.s1.set(t.rotate_left(37));
        s0.wrapping_mul(5).rotate_left(7).wrapping_mul(9)
    }

    pub fn next_f64(&self) -> f64 {
        (1.0 / (1u64 << 53) as f64) * (self.next_u64() >> 11) as f64
    }

    pub fn next_gauss(&self) -> f64 {
        let r = self.next_f64();
        let t = self.next_f64();
        f64::sqrt(-2.0 * f64::ln(1.0 - r)) * f64::sin((2.0 * f64::consts::PI) * t)
    }

    pub fn jump(&self) {
        const JUMP: [u64; 2] = [0xdf90_0294_d8f5_54a5, 0x1708_65df_4b32_01fc];

        let mut s0 = 0;
        let mut s1 = 0;
        for &jump in JUMP.iter() {
            for b in 0..64 {
                if jump & (1 << b) != 0 {
                    s0 ^= self.s0.get();
                    s1 ^= self.s1.get();
                }
                self.next_u64();
            }
        }
        self.s0.set(s0);
        self.s1.set(s1);
    }
}

#[test]
fn test() {
    let rng = Generator::new();
    assert!(rng.next_u64() == 10582614419484085930);
    assert!(rng.next_u64() == 16147916016143995109);
    assert!(rng.next_u64() == 5691192622506874316);
    assert!(rng.next_u64() == 14606526736076162211);
    rng.jump();
    assert!(rng.next_u64() == 4275479514889395181);
}
