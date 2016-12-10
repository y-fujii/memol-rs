// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.

#[derive(Debug)]
pub struct Definition {
	pub scores: Vec<(String, Box<Phrase>)>,
	pub values: Vec<(String, Box<Values>)>,
}

#[derive(Debug)]
pub enum Note {
	RelativeNote( i32, char, i32, i32 ),
	AbsoluteNote( char, i32, i32 ),
	Rest,
	Repeat,
	Octave( i32 ),
	Chord( Vec<Box<Note>> ),
	Group( Vec<(Box<Note>, i32)> ),
	Tie( Box<Note> ),
}

#[derive(Debug)]
pub enum Phrase {
	Repeat,
	Score( Vec<Box<Note>> ),
	Variable( String ),
	With( Box<Phrase>, char, Box<Phrase> ),
	Parallel( Vec<(Box<Phrase>, bool)> ),
	Sequence( Vec<Box<Phrase>> ),
}

#[derive(Debug)]
pub enum Values {
	Rest,
}
