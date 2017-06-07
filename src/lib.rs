// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
extern crate rand;
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
		let src = ::regex::Regex::new( r"(?s:/\*.*?\*/)" ).unwrap()
			.replace_all( src, |caps: &::regex::Captures|
				caps.get( 0 ).unwrap().as_str().chars().map( |c|
					if c == '\r' || c == '\n' { c } else { ' ' }
				).collect()
			);
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

pub const TICK: i64 = 480;

pub fn compile( src: &str ) -> Result<Vec<midi::Event>, misc::Error> {
	let tree = parser::parse( &src )?;
	let score_gen = scoregen::Generator::new( &tree );
	let value_gen = valuegen::Generator::new( &tree );

	let mut score_irs = Vec::new();
	for ch in 0 .. 16 {
		score_irs.push( score_gen.generate( &format!( "out.{}", ch ) )? );
	}

	let bgn = match value_gen.generate( "out.begin" )? {
		Some( ir ) => (ir.value( ratio::Ratio::zero() ) * TICK as f64).round() as i64,
		None       => 0,
	};
	let end = match value_gen.generate( "out.end" )? {
		Some( ir ) => (ir.value( ratio::Ratio::zero() ) * TICK as f64).round() as i64,
		None => {
			let v = score_irs.iter()
				.flat_map( |v| v.iter() )
				.flat_map( |v| v.notes.iter() )
				.map( |v| v.t1 )
				.max()
				.unwrap_or( ratio::Ratio::zero() );
			(v * TICK).round()
		}
	};

	let mut migen = midi::Generator::new( bgn, end, TICK );
	for (ch, score_ir) in score_irs.iter().enumerate() {
		if let Some( ref score_ir ) = *score_ir {
			let vel_ir = value_gen.generate( &format!( "out.{}.velocity", ch ) )?
				.unwrap_or( valuegen::Ir::Value(
					ratio::Ratio::zero(),
					ratio::Ratio::one(),
					ratio::Ratio::new( 5, 8 ),
					ratio::Ratio::new( 5, 8 ),
				) );
			let ofs_ir = value_gen.generate( &format!( "out.{}.offset", ch ) )?
				.unwrap_or( valuegen::Ir::Value(
					ratio::Ratio::zero(),
					ratio::Ratio::one(),
					ratio::Ratio::new( 0, 1 ),
					ratio::Ratio::new( 0, 1 ),
				) );
			migen.add_score( ch, &score_ir, &vel_ir, &ofs_ir );
		}
		for cc in 0 .. 128 {
			let value_ir = match value_gen.generate( &format!( "out.{}.cc{}", ch, cc ) )? {
				Some( v ) => v,
				None      => continue,
			};
			migen.add_cc( ch, cc, &value_ir );
		}
	}
	if let Some( tempo_ir ) = value_gen.generate( "out.tempo" )? {
		migen.add_tempo( &tempo_ir );
	}
	Ok( migen.generate()? )
}
