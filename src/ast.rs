// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use ratio;


#[derive(Clone, Debug)]
pub struct Ast<T: Clone> {
	pub ast: T,
	pub bgn: usize,
	pub end: usize,
}

#[derive(Clone, Debug)]
pub struct Definition<'a> {
	pub scores: collections::HashMap<String, Box<Ast<Score<'a>>>>,
	pub values: collections::HashMap<String, Box<Ast<ValueTrack>>>,
}

#[derive(Clone, Debug)]
pub enum Dir {
	Absolute( i32 ),
	Lower,
	Upper,
}

#[derive(Clone, Debug)]
pub enum Note<'a> {
	Note( Dir, char, i32, i32 ),
	Rest,
	Repeat( cell::RefCell<Option<&'a Ast<Note<'a>>>> ),
	Octave( i32 ),
	Chord( Vec<Box<Ast<Note<'a>>>> ),
	Group( Vec<(Box<Ast<Note<'a>>>, i32)> ),
	Tie( Box<Ast<Note<'a>>> ),
}

#[derive(Clone, Debug)]
pub enum Score<'a> {
	Score( Vec<Box<Ast<Note<'a>>>> ),
	Symbol( String ),
	With( Box<Ast<Score<'a>>>, char, Box<Ast<Score<'a>>> ),
	Parallel( Vec<Box<Ast<Score<'a>>>> ),
	Sequence( Vec<Box<Ast<Score<'a>>>> ),
	Stretch( Box<Ast<Score<'a>>>, ratio::Ratio ),
}

#[derive(Clone, Debug)]
pub enum Value {
	Value( ratio::Ratio, ratio::Ratio ),
	Group( Vec<(Box<Ast<Value>>, i32)> ),
}

#[derive(Clone, Debug)]
pub enum ValueTrack {
	ValueTrack( Vec<Box<Ast<Value>>> ),
	Symbol( String ),
	Sequence( Vec<Box<Ast<ValueTrack>>> ),
	Stretch( Box<Ast<ValueTrack>>, ratio::Ratio ),
}

impl<T: Clone> Ast<T> {
	pub fn new( bgn: usize, end: usize, ast: T ) -> Ast<T> {
		Ast{ ast: ast, bgn: bgn, end: end }
	}

	pub fn new_box( bgn: usize, end: usize, ast: T ) -> Box<Ast<T>> {
		Box::new( Self::new( bgn, end, ast ) )
	}
}
