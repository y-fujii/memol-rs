// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use ratio;


#[derive( Debug )]
pub struct Ast<T> {
	pub ast: T,
	pub bgn: usize,
	pub end: usize,
}

#[derive( Debug )]
pub struct Definition<'a> {
	pub scores: collections::HashMap<String, (path::PathBuf, Box<Ast<Score<'a>>>)>,
	pub values: collections::HashMap<String, (path::PathBuf, Box<Ast<Score<'a>>>)>,
}

#[derive( Copy, Clone, Debug )]
pub enum Dir {
	Lower,
	Upper,
}

#[derive( Debug )]
pub enum Note<'a> {
	Rest,
	Note( Dir, char, i64, i64 ),
	Value( Option<ratio::Ratio>, Option<ratio::Ratio> ),
	Repeat( cell::Cell<Option<&'a Ast<Note<'a>>>> ),
	Octave( i64 ),
	OctaveByNote( char, i64, i64 ),
	Chord( Vec<Box<Ast<Note<'a>>>> ),
	Group( Vec<(Box<Ast<Note<'a>>>, i64)> ),
	Tie( Box<Ast<Note<'a>>> ),
}

#[derive( Debug )]
pub enum Score<'a> {
	Score( Vec<Box<Ast<Note<'a>>>> ),
	Symbol( String ),
	Parallel( Vec<Box<Ast<Score<'a>>>> ),
	Sequence( Vec<Box<Ast<Score<'a>>>> ),
	With( Box<Ast<Score<'a>>>, char, Box<Ast<Score<'a>>> ),
	Repeat( Box<Ast<Score<'a>>>, i64 ),
	Stretch( Box<Ast<Score<'a>>>, ratio::Ratio ),
	Filter( Box<Ast<Score<'a>>>, Box<Ast<Score<'a>>> ),
	BinaryOp( Box<Ast<Score<'a>>>, Box<Ast<Score<'a>>>, BinaryOp ),
	Branch( Box<Ast<Score<'a>>>, Box<Ast<Score<'a>>>, Box<Ast<Score<'a>>> ),
	Slice( Box<Ast<Score<'a>>>, ratio::Ratio, ratio::Ratio ),
	Transpose( Box<Ast<Score<'a>>>, Box<Ast<Score<'a>>> ),
}

#[derive( Copy, Clone, Debug )]
pub enum BinaryOp {
	Add, Sub, Mul, Div, Eq, Ne, Le, Ge, Lt, Gt, Or,
}

impl<T> Ast<T> {
	pub fn new( bgn: usize, end: usize, ast: T ) -> Ast<T> {
		Ast{ ast: ast, bgn: bgn, end: end }
	}

	pub fn new_box( bgn: usize, end: usize, ast: T ) -> Box<Ast<T>> {
		Box::new( Self::new( bgn, end, ast ) )
	}
}
