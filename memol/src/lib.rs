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

pub struct Channel {
	pub score: scoregen::Ir,
	pub velocity: valuegen::Ir,
	pub offset: valuegen::Ir,
	pub duration: valuegen::Ir,
	pub ccs: Vec<(usize, valuegen::Ir)>,
}

pub struct Assembly {
	pub channels: Vec<(usize, Channel)>,
	pub tempo: valuegen::Ir,
	pub len: ratio::Ratio,
	pub bgn: ratio::Ratio,
	pub end: ratio::Ratio,
}

impl default::Default for Assembly {
	fn default() -> Self {
		Assembly{
			channels: Vec::new(),
			tempo: valuegen::Ir::Value(
				ratio::Ratio::zero(),
				ratio::Ratio::one(),
				ratio::Ratio::one(),
				ratio::Ratio::one(),
			),
			len: ratio::Ratio::zero(),
			bgn: ratio::Ratio::zero(),
			end: ratio::Ratio::zero(),
		}
	}
}

pub const TICK: i64 = 240;

pub fn compile( src: &str ) -> Result<Assembly, misc::Error> {
	let tree = parser::parse( &src )?;
	let score_gen = scoregen::Generator::new( &tree );
	let value_gen = valuegen::Generator::new( &tree );

	let mut scores = Vec::new();
	for ch in 0 .. 16 {
		if let Some( ir ) = score_gen.generate( &format!( "out.{}", ch ) )? {
			scores.push( (ch, ir) );
		}
	}

	let mut channels = Vec::new();
	for (ch, score) in scores.into_iter() {
		let vel = value_gen.generate( &format!( "out.{}.velocity", ch ) )?
			.unwrap_or( valuegen::Ir::Value(
				ratio::Ratio::zero(),
				ratio::Ratio::one(),
				ratio::Ratio::new( 5, 8 ),
				ratio::Ratio::new( 5, 8 ),
			) );
		let ofs = value_gen.generate( &format!( "out.{}.offset", ch ) )?
			.unwrap_or( valuegen::Ir::Value(
				ratio::Ratio::zero(),
				ratio::Ratio::one(),
				ratio::Ratio::new( 0, 1 ),
				ratio::Ratio::new( 0, 1 ),
			) );
		let dur = value_gen.generate( &format!( "out.{}.duration", ch ) )?
			.unwrap_or( valuegen::Ir::Symbol( "note.len".into() ) );
		let mut ccs = Vec::new();
		for cc in 0 .. 128 {
			if let Some( ir ) = value_gen.generate( &format!( "out.{}.cc{}", ch, cc ) )? {
				ccs.push( (cc, ir) );
			}
		}
		channels.push( (ch, Channel{ score: score, velocity: vel, offset: ofs, duration: dur, ccs: ccs }) );
	}

	let tempo = value_gen.generate( "out.tempo" )?
		.unwrap_or( valuegen::Ir::Value(
			ratio::Ratio::zero(),
			ratio::Ratio::one(),
			ratio::Ratio::one(),
			ratio::Ratio::one(),
		) );

	let len = channels.iter()
		.flat_map( |&(_, ref v)| v.score.notes.iter() )
		.map( |v| v.t1 )
		.max()
		.unwrap_or( ratio::Ratio::zero() );

	let mut evaluator = valuegen::Evaluator::new();
	let bgn = match value_gen.generate( "out.begin" )? {
		Some( ir ) => (evaluator.eval( &ir, ratio::Ratio::zero() ) * TICK as f64).round() as i64,
		None       => 0,
	};
	let end = match value_gen.generate( "out.end" )? {
		Some( ir ) => (evaluator.eval( &ir, ratio::Ratio::zero() ) * TICK as f64).round() as i64,
		None       => (len * TICK).round()
	};

	Ok( Assembly{
		channels: channels,
		tempo: tempo,
		len: len,
		bgn: ratio::Ratio::new( bgn, TICK ),
		end: ratio::Ratio::new( end, TICK ),
	} )
}

pub fn assemble( src: &Assembly ) -> Result<Vec<midi::Event>, misc::Error> {
	let mut rng = rand::ChaChaRng::new_unseeded();

	let bgn = (src.bgn * TICK).round();
	let end = (src.end * TICK).round();
	let mut migen = midi::Generator::new( &mut rng, bgn, end, TICK );
	for &(ch, ref irs) in src.channels.iter() {
		migen.add_score( ch, &irs.score, &irs.velocity, &irs.offset, &irs.duration );
		for &(cc, ref ir) in irs.ccs.iter() {
			migen.add_cc( ch, cc, &ir );
		}
	}
	migen.add_tempo( &src.tempo );
	Ok( migen.generate()? )
}
