// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;


#[derive(Debug)]
pub struct Error {
	pub text: String,
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

pub fn error<T, U: From<Error>>( text: &str ) -> Result<T, U> {
	Err( From::from( Error{ text: text.into() } ) )
}

#[allow( deprecated )]
pub fn idiv<T: Copy + cmp::Ord + ops::Sub<Output = T> + ops::Mul<Output = T> + ops::Div<Output = T> + num::One>( x: T, y: T ) -> T {
	let r = x / y;
	if r * y <= x { r } else { r - T::one() }
}

#[allow( deprecated )]
pub fn imod<T: Copy + cmp::Ord + ops::Sub<Output = T> + ops::Mul<Output = T> + ops::Div<Output = T> + num::One>( x: T, y: T ) -> T {
	x - y * idiv( x, y )
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

#[macro_export]
macro_rules! c_str {
	($e: tt) => (
		concat!( $e, "\0" ).as_ptr() as *const os::raw::c_char
	);
	($e: tt, $($arg: tt)*) => (
		format!( concat!( $e, "\0" ), $($arg)* ).as_ptr() as *const os::raw::c_char
	)
}
