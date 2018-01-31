// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
extern crate regex;
extern crate lalrpop_util;
extern crate libloading;
#[macro_use]
pub mod misc;
pub mod random;
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
	use std::io::prelude::*;

	include!( "parser.rs" );

	// XXX
	fn remove_comments( src: &str ) -> borrow::Cow<str> {
		use ::regex::*;
		thread_local!( static RE: Regex = Regex::new( r"(?s:/\*.*?\*/)" ).unwrap() );
		RE.with( |re|
			re.replace_all( src, |caps: &Captures|
				caps.get( 0 ).unwrap().as_str().chars().map( |c|
					if c == '\r' || c == '\n' { c } else { ' ' }
				).collect()
			)
		)
	}

	pub fn parse<'a>( path: &path::Path ) -> Result<Definition<'a>, ::misc::Error> {
		let mut buf = String::new();
		fs::File::open( path )
			.map_err( |e| misc::Error::new( path, 0, format!( "{}", e ) ) )?
			.read_to_string( &mut buf )
			.map_err( |e| misc::Error::new( path, 0, format!( "{}", e ) ) )?;

		match parse_definition( path, &remove_comments( &buf ) ) {
			Ok ( v ) => Ok( v ),
			Err( e ) => {
				match e {
					ParseError::InvalidToken{ location: i } |
					ParseError::UnrecognizedToken{ token: Some( (i, _, _) ), .. } |
					ParseError::ExtraToken{ token: (i, _, _) } =>
						::misc::error( path, i, "unexpected token." ),
					ParseError::UnrecognizedToken{ token: None, .. } =>
						::misc::error( path, buf.len(), "unexpected EOF." ),
					ParseError::User{ error: err } =>
						Err( err ),
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
	pub pitch: valuegen::Ir,
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

pub fn compile( src: &path::Path ) -> Result<Assembly, misc::Error> {
	let tree = parser::parse( src )?;
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
		let velocity = value_gen.generate( &format!( "out.{}.velocity", ch ) )?
			.unwrap_or( valuegen::Ir::Value(
				ratio::Ratio::zero(),
				ratio::Ratio::one(),
				ratio::Ratio::new( 5, 8 ),
				ratio::Ratio::new( 5, 8 ),
			) );
		let offset = value_gen.generate( &format!( "out.{}.offset", ch ) )?
			.unwrap_or( valuegen::Ir::Value(
				ratio::Ratio::zero(),
				ratio::Ratio::one(),
				ratio::Ratio::new( 0, 1 ),
				ratio::Ratio::new( 0, 1 ),
			) );
		let duration = value_gen.generate( &format!( "out.{}.duration", ch ) )?
			.unwrap_or( valuegen::Ir::Symbol( "note.len".into() ) );
		let pitch = value_gen.generate( &format!( "out.{}.pitch", ch ) )?
			.unwrap_or( valuegen::Ir::Value(
				ratio::Ratio::zero(),
				ratio::Ratio::one(),
				ratio::Ratio::new( 0, 1 ),
				ratio::Ratio::new( 0, 1 ),
			) );
		let mut ccs = Vec::new();
		for cc in 0 .. 128 {
			if let Some( ir ) = value_gen.generate( &format!( "out.{}.cc{}", ch, cc ) )? {
				ccs.push( (cc, ir) );
			}
		}
		channels.push( (ch, Channel{
			score: score,
			velocity: velocity,
			offset: offset,
			duration: duration,
			pitch: pitch,
			ccs: ccs,
		}) );
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
	let mut rng = random::Generator::new();

	let bgn = (src.bgn * TICK).round();
	let end = (src.end * TICK).round();
	let mut migen = midi::Generator::new( &mut rng, bgn, end, TICK );
	for &(ch, ref irs) in src.channels.iter() {
		migen.add_score( ch, &irs.score, &irs.velocity, &irs.offset, &irs.duration );
		migen.add_pitch( ch, &irs.pitch );
		for &(cc, ref ir) in irs.ccs.iter() {
			migen.add_cc( ch, cc, &ir );
		}
	}
	migen.add_tempo( &src.tempo );
	Ok( migen.generate()? )
}
