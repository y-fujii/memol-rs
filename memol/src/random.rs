// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;


pub type Generator = XoroShiro128Plus;

// Xoroshiro128+ PRNG, by David Blackman and Sebastiano Vigna.
// ref. <http://vigna.di.unimi.it/xorshift/xoroshiro128plus.c>
pub struct XoroShiro128Plus {
	s0: cell::Cell<u64>,
	s1: cell::Cell<u64>,
}

impl XoroShiro128Plus {
	pub fn new() -> Self {
		Self{
			s0: cell::Cell::new( 0x243f_6a88_85a3_08d3 ), // OEIS A062964.
			s1: cell::Cell::new( 0x93c4_67e3_7db0_c7a4 ), // OEIS A170874.
		}
	}

	pub fn next_u64( &self ) -> u64 {
		let s0 = self.s0.get();
		let s1 = self.s1.get();
		let t = s0 ^ s1;
		self.s0.set( s0.rotate_left( 55 ) ^ t ^ (t << 14) );
		self.s1.set( t.rotate_left( 36 ) );
		u64::wrapping_add( s0, s1 )
	}

	pub fn next_f64( &self ) -> f64 {
		(1.0 / (1u64 << 53) as f64) * (self.next_u64() >> 11) as f64
	}

	pub fn next_gauss( &self ) -> f64 {
		let r = self.next_f64();
		let t = self.next_f64();
		f64::sqrt( -2.0 * f64::ln( 1.0 - r ) ) * f64::sin( (2.0 * f64::consts::PI) * t )
	}

	pub fn jump( &self ) {
		const JUMP: [u64; 2] = [ 0xbeac_0467_eba5_facb, 0xd86b_048b_86aa_9922 ];

		let mut s0 = 0;
		let mut s1 = 0;
		for &jump in JUMP.iter() {
			for b in 0 .. 64 {
				if jump & (1 << b) != 0 {
					s0 ^= self.s0.get();
					s1 ^= self.s1.get();
				}
				self.next_u64();
			}
		}
		self.s0.set( s0 );
		self.s1.set( s1 );
	}
}


#[test]
fn test() {
	let rng = Generator::new();
	assert!( rng.next_u64() == 13259673089262997623 );
	assert!( rng.next_u64() == 11416876112584488370 );
	assert!( rng.next_u64() ==  2822159522531543094 );
	assert!( rng.next_u64() ==  7148299523015547248 );
	let rng = Generator::new();
	rng.jump();
	assert!( rng.next_u64() ==  6516743372915791242 );
}
