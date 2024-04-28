// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;

// The PRNG that aims to generate independent sequences for different seeds, using simplified
// version of LXM (doi:10.1145/3485525).
pub struct Generator {
    a: u64,
    s: cell::Cell<u64>,
}

impl Generator {
    pub fn new() -> Self {
        let seed = 0;
        Generator {
            a: (seed << 1) | 1,
            s: cell::Cell::new(0),
        }
    }

    pub fn next_u64(&self) -> u64 {
        let z = self.s.get().wrapping_mul(0xd1342543de82ef95) + self.a;
        self.s.set(z);
        let z = (z ^ (z >> 32)).wrapping_mul(0xdaba0b6eb09322e3);
        let z = (z ^ (z >> 32)).wrapping_mul(0xdaba0b6eb09322e3);
        z ^ (z >> 32)
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
