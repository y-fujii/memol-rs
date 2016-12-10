// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD License.
use std::*;


#[derive(Copy, Clone, Debug)]
pub struct Ratio {
	pub y: i32,
	pub x: i32,
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
		if self.x * other.x < 0 {
			(other.y * self.x).cmp( &(self.y * other.x) )
		}
		else {
			(self.y * other.x).cmp( &(other.y * self.x) )
		}
    }
}

impl ops::Add for Ratio {
	type Output = Ratio;

	fn add( self, other: Ratio ) -> Ratio {
		Ratio {
			y: self.y * other.x + self.x * other.y,
			x: self.x * other.x,
		}
	}
}

impl ops::Add<i32> for Ratio {
	type Output = Ratio;

	fn add( self, other: i32 ) -> Ratio {
		Ratio {
			y: self.y + other * self.x,
			x: self.x,
		}
	}
}

impl ops::Sub for Ratio {
	type Output = Ratio;

	fn sub( self, other: Ratio ) -> Ratio {
		Ratio {
			y: self.y * other.x - self.x * other.y,
			x: self.x * other.x,
		}
	}
}

impl ops::Mul for Ratio {
	type Output = Ratio;

	fn mul( self, other: Ratio ) -> Ratio {
		Ratio {
			y: self.y * other.y,
			x: self.x * other.x,
		}
	}
}

impl ops::Div<i32> for Ratio {
	type Output = Ratio;

	fn div( self, other: i32 ) -> Ratio {
		Ratio {
			y: self.y,
			x: self.x * other,
		}
	}
}

impl Ratio {
	pub fn new( y: i32, x: i32 ) -> Ratio {
		Ratio{ y: y, x: x }
	}
}
