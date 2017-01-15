// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![feature( zero_one )]
extern crate regex;
extern crate lalrpop_util;
#[macro_use]
pub mod misc;
pub mod ratio;
pub mod ast;
//pub mod parser;
pub mod irgen;
pub mod midi;
pub mod jack;
pub mod player;
pub mod notify;
use std::*;


pub mod parser {
	include!( "parser.rs" );

	pub fn parse( src: &str ) -> Result<Definition, ::misc::Error> {
		// XXX
		let src = ::regex::Regex::new( r"(?s:/\*.*?\*/)" ).unwrap().replace_all( src, "" );
		match parse_definition( &src ) {
			Ok ( v ) => Ok( v ),
			Err( e ) => ::misc::error( &format!( "{:?}", e ) ),
		}
	}
}

pub fn compile( src: &str ) -> Result<Vec<midi::Event>, misc::Error> {
	let tree = parser::parse( &src )?;
	let irgen = irgen::Generator::new( &tree );
	let mut migen = midi::Generator::new();
	for ch in 0 .. 16 {
		if let Some( ir ) = irgen.generate( &format!( "out.{}", ch ) )? {
			migen = migen.add_score( ch, &ir );
		}
	}
	Ok( migen.generate() )
}
