// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD License.
use std::*;


#[derive(Debug)]
pub struct Error {
	text: String,
}

impl Error {
	pub fn new<T>( text: &str ) -> Result<T, Error> {
		Err( Error{ text: text.into() } )
	}
}

impl fmt::Display for Error {
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
		f.write_str( &self.text )
	}
}

impl error::Error for Error {
	fn description( &self ) -> &str {
		&self.text
	}
}

// sign( gcd( y, x ) ) == sign( x )
pub fn gcd( y: i64, x: i64 ) -> i64 {
	let s = x < 0;
	let mut y = y.abs();
	let mut x = x.abs();
	while x != 0 {
		let t = y % x;
		y = x;
		x = t;
	}
	if s { -y } else { y }
}

pub fn bsearch_boundary<T, F: FnMut( &T ) -> bool>( xs: &[T], mut f: F ) -> usize {
	/*
		semantically equivalent to:
			for (i, x) in xs.iter().enumerate() {
				if !f( x ) {
					return i;
				}
			}
			return xs.len();

		invariants:
			f( xs[lo - 1] ) == true
			f( xs[hi    ] ) == false
	*/
	let mut lo = 0;
	let mut hi = xs.len();
	while lo < hi {
		let mi = (lo + hi) / 2;
		if f( &xs[mi] ) {
			lo = mi + 1;
		}
		else {
			hi = mi;
		}
	}
	lo
}
