// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

// The PRNG that aims to generate independent sequences for different seeds, using simplified
// version of LXM (doi:10.1145/3485525).
pub struct Generator {
    a: u64,
    s: cell::Cell<u64>,
}

fn lea64(z: u64) -> u64 {
    let z = (z ^ (z >> 32)).wrapping_mul(0xdaba0b6eb09322e3);
    let z = (z ^ (z >> 32)).wrapping_mul(0xdaba0b6eb09322e3);
    z ^ (z >> 32)
}

impl Generator {
    const M: u64 = 0xd1342543de82ef95;

    pub fn new(seed: u64) -> Self {
        assert!(seed < Self::M - 1 >> 1);
        let seed = seed << 1 | 1;
        Generator {
            a: seed,
            s: cell::Cell::new(lea64(seed)),
        }
    }

    pub fn next_u64(&self) -> u64 {
        let z = self.s.get();
        self.s.set(z.wrapping_mul(Self::M) + self.a);
        lea64(z)
    }

    pub fn next_f64(&self) -> f64 {
        (1.0 / (1u64 << 53) as f64) * (self.next_u64() >> 11) as f64
    }

    pub fn next_gauss(&self) -> f64 {
        let r = self.next_f64();
        let t = self.next_f64();
        f64::sqrt(-2.0 * f64::ln(1.0 - r)) * f64::sin(f64::consts::TAU * t)
    }
}
