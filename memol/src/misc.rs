// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use std::io::Read;


pub trait One {
	fn one() -> Self;
}

impl One for i32 {
	fn one() -> Self {
		1
	}
}

impl One for i64 {
	fn one() -> Self {
		1
	}
}

#[derive(Debug)]
pub struct UniqueIterator<T: Iterator> {
	prev: Option<T::Item>,
	iter: T,
}

impl<T: Iterator> Iterator for UniqueIterator<T> where T::Item: PartialEq {
	type Item = T::Item;

	fn next( &mut self ) -> Option<Self::Item> {
		if let None = self.prev {
			return None;
		}
		loop {
			let next = self.iter.next();
			if next != self.prev {
				return mem::replace( &mut self.prev, next );
			}
		}
	}
}

pub trait IteratorEx<T: Iterator> {
	fn unique( self ) -> UniqueIterator<T>;
}

impl<T: Iterator> IteratorEx<T> for T {
	fn unique( mut self ) -> UniqueIterator<T> {
		let prev = self.next();
		UniqueIterator{ prev: prev, iter: self }
	}
}

pub fn idiv<T: Copy + cmp::Ord + ops::Sub<Output = T> + ops::Mul<Output = T> + ops::Div<Output = T> + One>( x: T, y: T ) -> T {
	let r = x / y;
	if r * y <= x { r } else { r - T::one() }
}

pub fn imod<T: Copy + cmp::Ord + ops::Sub<Output = T> + ops::Mul<Output = T> + ops::Div<Output = T> + One>( x: T, y: T ) -> T {
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

#[derive(Debug)]
pub struct Error {
	pub path: path::PathBuf,
	pub index: usize,
	pub message: String,
}

impl fmt::Display for Error {
	fn fmt( &self, _: &mut fmt::Formatter ) -> fmt::Result {
		panic!();
	}
}

impl error::Error for Error {
	fn description( &self ) -> &str {
		panic!();
	}
}

impl Error {
	pub fn new<T: convert::Into<String>>( path: &path::Path, idx: usize, msg: T ) -> Self {
		Error{ path: path.to_owned(), index: idx, message: msg.into() }
	}

	pub fn message( &self ) -> String {
		let path = self.path.to_string_lossy();
		let mut buf = String::new();
		match fs::File::open( &*self.path ).and_then( |mut f| f.read_to_string( &mut buf ) ) {
			Ok( _ ) =>  {
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
					};
				}
				format!( "{}:{}:{} {}", path, row, col, self.message )
			},
			Err( _ ) => {
				format!( "{} {}", path, self.message )
			},
		}
	}
}

pub fn error<T: convert::Into<String>, U, V: From<Error>>( path: &path::Path, idx: usize, msg: T ) -> Result<U, V> {
	Err( From::from( Error::new( path, idx, msg ) ) )
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
