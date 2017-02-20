// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![feature( zero_one )]
extern crate regex;
extern crate lalrpop_util;
#[macro_use]
pub mod misc;
pub mod ratio;
pub mod ast;
//pub mod parser;
pub mod scoregen;
pub mod valuegen;
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
			Err( e ) => {
				use ::lalrpop_util::ParseError;
				match e {
					ParseError::InvalidToken{ location: loc } |
					ParseError::UnrecognizedToken{ token: Some( (loc, _, _) ), .. } |
					ParseError::ExtraToken{ token: (loc, _, _) } =>
						::misc::error( loc, "unexpected token." ),
					ParseError::UnrecognizedToken{ token: None, .. } =>
						::misc::error( src.len(), "unexpected EOF." ),
					ParseError::User{ error: (loc, msg) } =>
						::misc::error( loc, msg ),
				}
			}
		}
	}
}

pub fn compile( src: &str ) -> Result<(Vec<midi::Event>, Vec<ratio::Ratio>), misc::Error> {
	let tree = parser::parse( &src )?;
	let score_gen = scoregen::Generator::new( &tree );
	let value_gen = valuegen::Generator::new( &tree );
	let mut migen = midi::Generator::new();
	for ch in 0 .. 16 {
		if let Some( score_ir ) = score_gen.generate( &format!( "out.{}", ch ) )? {
			let value_ir = match value_gen.generate( &format!( "out.{}.velocity", ch ) )? {
				Some( v ) => v,
				None => valuegen::Ir{ values: vec![ valuegen::FlatValue{
					bgn: -ratio::Ratio::inf(),
					end:  ratio::Ratio::inf(),
					value_bgn: ratio::Ratio::new( 6, 127 ), // XXX
					value_end: ratio::Ratio::new( 6, 127 ), // XXX
				} ] },
			};

			migen = migen.add_score( ch, &score_ir, &value_ir );
		}
	}
	Ok( migen.generate() )
}
