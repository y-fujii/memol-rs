// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use misc;


#[derive(Copy, Clone, Debug)]
pub struct Ratio {
	pub y: i64,
	pub x: i64,
}

impl cmp::PartialEq for Ratio {
	fn eq( &self, other: &Self ) -> bool {
		self.y * other.x == other.y * self.x
	}
}

impl cmp::Eq for Ratio {
}

impl PartialOrd for Ratio {
	fn partial_cmp( &self, other: &Ratio ) -> Option<cmp::Ordering> {
		Some( self.cmp( other ) )
	}
}

impl cmp::Ord for Ratio {
    fn cmp( &self, other: &Ratio ) -> cmp::Ordering {
		let lhs = self.y * other.x;
		let rhs = other.y * self.x;
		// 0 denominator is interpreted as +0.
		if self.x * other.x < 0 {
			rhs.cmp( &lhs )
		}
		else {
			lhs.cmp( &rhs )
		}
    }
}

impl ops::Neg for Ratio {
	type Output = Ratio;

	fn neg( self ) -> Ratio {
		Ratio{ y: -self.y, x: self.x }
	}
}

impl ops::Add for Ratio {
	type Output = Ratio;

	fn add( self, other: Ratio ) -> Ratio {
		Ratio::new(
			self.y * other.x + self.x * other.y,
			self.x * other.x,
		)
	}
}

impl ops::Add<i64> for Ratio {
	type Output = Ratio;

	fn add( self, other: i64 ) -> Ratio {
		Ratio::new(
			self.y + other * self.x,
			self.x,
		)
	}
}

impl ops::Sub for Ratio {
	type Output = Ratio;

	fn sub( self, other: Ratio ) -> Ratio {
		Ratio::new(
			self.y * other.x - self.x * other.y,
			self.x * other.x,
		)
	}
}

impl ops::Sub<i64> for Ratio {
	type Output = Ratio;

	fn sub( self, other: i64 ) -> Ratio {
		Ratio::new(
			self.y - other * self.x,
			self.x,
		)
	}
}

impl ops::Sub<Ratio> for i64 {
	type Output = Ratio;

	fn sub( self, other: Ratio ) -> Ratio {
		Ratio::new(
			self * other.x - other.y,
			other.x,
		)
	}
}

impl ops::Mul for Ratio {
	type Output = Ratio;

	fn mul( self, other: Ratio ) -> Ratio {
		Ratio::new(
			self.y * other.y,
			self.x * other.x,
		)
	}
}

impl ops::Mul<i64> for Ratio {
	type Output = Ratio;

	fn mul( self, other: i64 ) -> Ratio {
		Ratio::new(
			self.y * other,
			self.x,
		)
	}
}

impl ops::Div for Ratio {
	type Output = Ratio;

	fn div( self, other: Ratio ) -> Ratio {
		Ratio::new(
			self.y * other.x,
			self.x * other.y,
		)
	}
}

impl ops::Div<i64> for Ratio {
	type Output = Ratio;

	fn div( self, other: i64 ) -> Ratio {
		Ratio::new(
			self.y,
			self.x * other,
		)
	}
}

impl Ratio {
	pub fn new( y: i64, x: i64 ) -> Ratio {
		let t = misc::gcd( y, x );
		Ratio{
			y: y / t,
			x: x / t,
		}
	}

	pub fn zero() -> Ratio {
		Ratio{ y: 0, x: 1 }
	}

	pub fn one() -> Ratio {
		Ratio{ y: 1, x: 1 }
	}

	pub fn inf() -> Ratio {
		Ratio{ y: 1, x: 0 }
	}

	pub fn floor( self ) -> i64 {
		misc::idiv( self.y, self.x )
	}

	pub fn ceil( self ) -> i64 {
		misc::idiv( self.y + self.x - 1, self.x )
	}

	pub fn round( self ) -> i64 {
		misc::idiv( self.y * 2 + self.x, self.x * 2 )
	}

	pub fn to_float( self ) -> f64 {
		self.y as f64 / self.x as f64
	}
}
