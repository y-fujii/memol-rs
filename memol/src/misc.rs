// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use std::io::Read;


pub fn idiv( x: i64, y: i64 ) -> i64 {
	let r = x / y;
	if r * y <= x { r } else { r - 1 }
}

pub fn imod( x: i64, y: i64 ) -> i64 {
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

// semantically equivalent to:
//     for (i, x) in xs.iter().enumerate() {
//         if !f( x ) {
//             return i;
//         }
//     }
//     return xs.len();
pub fn bsearch_boundary<T, F: FnMut( &T ) -> bool>( xs: &[T], mut f: F ) -> usize {
	// invariants:
	//     f( xs[lo - 1] ) == true
	//     f( xs[hi    ] ) == false
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

pub fn u16_to_bytes_be( i: u16 ) -> [u8; 2] {
	[ (i >> 8) as u8, i as u8 ]
}

pub fn u32_to_bytes_be( i: u32 ) -> [u8; 4] {
	[ (i >> 24) as u8, (i >> 16) as u8, (i >> 8) as u8, i as u8 ]
}

#[derive( Debug )]
pub struct Error {
	pub path: path::PathBuf,
	pub index: usize,
	pub message: String,
}

impl fmt::Display for Error {
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
		let path = self.path.to_string_lossy();
		let mut buf = String::new();
		match fs::File::open( &*self.path ).and_then( |mut f| f.read_to_string( &mut buf ) ) {
			Ok( _ ) => {
				let mut row = 0;
				let mut col = 0;
				for c in buf.chars().take( self.index ) {
					match c {
						'\r' => (),
						'\n' => {
							row += 1;
							col = 0;
						},
						_ => {
							col += 1;
						},
					}
				}
				write!( f, "{}:{}:{}: {}", path, row, col, self.message )
			},
			Err( _ ) => {
				write!( f, "{}: {}", path, self.message )
			},
		}
	}
}

impl error::Error for Error {
	fn description( &self ) -> &str {
		panic!();
	}
}

impl Error {
	pub fn new<T: Into<String>>( path: &path::Path, idx: usize, msg: T ) -> Self {
		Error{ path: path.to_owned(), index: idx, message: msg.into() }
	}
}

pub fn error<T: convert::Into<String>, U, V: From<Error>>( path: &path::Path, idx: usize, msg: T ) -> Result<U, V> {
	Err( From::from( Error::new( path, idx, msg ) ) )
}
