// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;


#[derive(Clone, Debug)]
pub struct Ast<T: Clone> {
	pub ast: T,
	pub bgn: usize,
	pub end: usize,
}

#[derive(Clone, Debug)]
pub struct Definition {
	pub scores: collections::HashMap<String, Box<Ast<Score>>>,
	pub values: collections::HashMap<String, Box<Ast<ValueTrack>>>,
}

#[derive(Clone, Debug)]
pub enum Dir {
	Absolute( i32 ),
	Lower,
	Upper,
}

#[derive(Clone, Debug)]
pub enum Note {
	Note( Dir, char, i32, i32 ),
	Rest,
	Repeat,
	Mark,
	Octave( i32 ),
	Chord( Vec<Box<Ast<Note>>> ),
	Group( Vec<(Box<Ast<Note>>, i32)> ),
	Tie( Box<Ast<Note>> ),
}

#[derive(Clone, Debug)]
pub enum Score {
	Score( Vec<Box<Ast<Note>>> ),
	Symbol( String ),
	With( Box<Ast<Score>>, char, Box<Ast<Score>> ),
	Parallel( Vec<Box<Ast<Score>>> ),
	Sequence( Vec<Box<Ast<Score>>> ),
}

#[derive(Clone, Debug)]
pub enum Value {
	Value( i32, i32 ),
	Group( Vec<(Box<Ast<Value>>, i32)> ),
}

#[derive(Clone, Debug)]
pub enum ValueTrack {
	ValueTrack( Vec<Box<Ast<Value>>> ),
	Symbol( String ),
	Sequence( Vec<Box<Ast<ValueTrack>>> ),
}

impl<T: Clone> Ast<T> {
	pub fn new( bgn: usize, end: usize, ast: T ) -> Ast<T> {
		Ast{ ast: ast, bgn: bgn, end: end }
	}

	pub fn new_box( bgn: usize, end: usize, ast: T ) -> Box<Ast<T>> {
		Box::new( Self::new( bgn, end, ast ) )
	}
}
