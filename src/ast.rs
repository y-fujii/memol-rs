// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
use std::*;

#[derive(Debug)]
pub struct Definition {
	pub scores: collections::HashMap<String, Box<Score>>,
	pub values: collections::HashMap<String, Box<Value>>,
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
	Octave( i32 ),
	Chord( Vec<Box<Note>> ),
	Group( Vec<(Box<Note>, i32)> ),
	Tie( Box<Note> ),
}

#[derive(Clone, Debug)]
pub enum Score {
	Score( Vec<Box<Note>> ),
	Variable( String ),
	With( Box<Score>, char, Box<Score> ),
	Parallel( Vec<Box<Score>> ),
	Sequence( Vec<Box<Score>> ),
}

#[derive(Clone, Debug)]
pub enum Value {
	Rest,
}
