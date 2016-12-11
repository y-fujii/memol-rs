// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD License.
use std::*;


// sign( gcd( y, x ) ) == sign( x )
pub fn gcd( y: i32, x: i32 ) -> i32 {
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

/*
pub fn lower_bound<T, U, F: FnMut( &T, &U ) -> bool>( xs: &Vec<T>, y: &U, mut cmp: F ) -> usize {
	0
}
*/
