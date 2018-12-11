mod score;
mod value;
mod eval;
use std::*;
use crate::ast;
use crate::ratio::Ratio;
use crate::random;
pub use self::score::*;
pub use self::value::*;
pub use self::eval::*;


#[derive( Clone, Debug )]
pub struct FlatNote {
	pub t0: Ratio,
	pub t1: Ratio,
	pub nnum: Option<i64>,
}

pub struct Span<'a> {
	t0: Ratio,
	dt: Ratio,
	tied: bool,
	syms: &'a collections::HashMap<char, &'a [FlatNote]>,
	path: &'a path::Path,
}

pub struct Generator<'a> {
	rng: &'a random::Generator,
	defs: &'a ast::Definition<'a>,
	syms: Vec<(char, Vec<FlatNote>)>,
}

impl<'a> Generator<'a> {
	pub fn new( rng: &'a random::Generator, defs: &'a ast::Definition<'a> ) -> Generator<'a> {
		let syms = vec![ ('*', vec![
			FlatNote{ t0: -Ratio::inf(), t1: Ratio::inf(), nnum: Some( 69 ) },
			FlatNote{ t0: -Ratio::inf(), t1: Ratio::inf(), nnum: Some( 71 ) },
			FlatNote{ t0: -Ratio::inf(), t1: Ratio::inf(), nnum: Some( 60 ) },
			FlatNote{ t0: -Ratio::inf(), t1: Ratio::inf(), nnum: Some( 62 ) },
			FlatNote{ t0: -Ratio::inf(), t1: Ratio::inf(), nnum: Some( 64 ) },
			FlatNote{ t0: -Ratio::inf(), t1: Ratio::inf(), nnum: Some( 65 ) },
			FlatNote{ t0: -Ratio::inf(), t1: Ratio::inf(), nnum: Some( 67 ) },
		]) ];

		Generator{
			rng: rng,
			defs: defs,
			syms: syms,
		}
	}
}
