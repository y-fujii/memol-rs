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

pub fn compile( src: &str ) -> Result<(Vec<midi::Event>, Option<(ratio::Ratio, ratio::Ratio)>), misc::Error> {
	let tree = parser::parse( &src )?;
	let score_gen = scoregen::Generator::new( &tree );
	let value_gen = valuegen::Generator::new( &tree );

	let bgn_ir = value_gen.generate( "out.begin" )?;
	let end_ir = value_gen.generate( "out.end"   )?;
	let range = match (bgn_ir, end_ir) {
		(Some( bgn ), Some( end )) => Some( (
			bgn.get_value( ratio::Ratio::zero() ),
			end.get_value( ratio::Ratio::zero() ),
		) ),
		_ => None,
	};

	let mut migen = midi::Generator::new();
	for ch in 0 .. 16 {
		if let Some( score_ir ) = score_gen.generate( &format!( "out.{}", ch ) )? {
			let value_ir = match value_gen.generate( &format!( "out.{}.velocity", ch ) )? {
				Some( v ) => v,
				None => valuegen::Ir{ values: vec![ valuegen::FlatValue{
					t0: -ratio::Ratio::inf(),
					t1:  ratio::Ratio::inf(),
					v0: ratio::Ratio::new( 6, 127 ), // XXX
					v1: ratio::Ratio::new( 6, 127 ), // XXX
				} ] },
			};
			migen = migen.add_score( ch, &score_ir, &value_ir );
		}
		for cc in 0 .. 127 {
			let value_ir = match value_gen.generate( &format!( "out.{}.cc{}", ch, cc ) )? {
				Some( v ) => v,
				None      => continue,
			};
			migen = migen.add_cc( ch, cc, &value_ir );
		}
	}
	Ok( (migen.generate(), range) )
}
