// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
extern crate regex;
extern crate lalrpop_util;
#[macro_use]
pub mod misc;
pub mod random;
pub mod ratio;
pub mod ast;
pub mod generator;
pub mod midi;
pub mod smf;
use std::*;


pub mod parser {
	use std::io::prelude::*;

	include!( concat!( env!( "OUT_DIR" ), "/parser.rs" ) );

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
	pub score: generator::ScoreIr,
	pub velocity: generator::ValueIr,
	pub offset: generator::ValueIr,
	pub duration: generator::ValueIr,
	pub pitch: generator::ValueIr,
	pub ccs: Vec<(usize, generator::ValueIr)>,
}

pub struct Assembly {
	pub channels: Vec<(usize, Channel)>,
	pub tempo: generator::ValueIr,
	pub len: ratio::Ratio,
	pub bgn: ratio::Ratio,
	pub end: ratio::Ratio,
}

impl default::Default for Assembly {
	fn default() -> Self {
		Assembly{
			channels: Vec::new(),
			tempo: generator::ValueIr::Value(
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

pub fn compile( rng: &random::Generator, src: &path::Path ) -> Result<Assembly, misc::Error> {
	let tree = parser::parse( src )?;
	let gen = generator::Generator::new( rng, &tree );

	let mut scores = Vec::new();
	for ch in 0 .. 16 {
		if let Some( ir ) = gen.generate_score( &format!( "out.{}", ch ) )? {
			scores.push( (ch, ir) );
		}
	}

	let mut channels = Vec::new();
	for (ch, score) in scores.into_iter() {
		let velocity = gen.generate_value( &format!( "out.{}.velocity", ch ) )?
			.unwrap_or( generator::ValueIr::Value(
				ratio::Ratio::zero(),
				ratio::Ratio::one(),
				ratio::Ratio::new( 5, 8 ),
				ratio::Ratio::new( 5, 8 ),
			) );
		let offset = gen.generate_value( &format!( "out.{}.offset", ch ) )?
			.unwrap_or( generator::ValueIr::Value(
				ratio::Ratio::zero(),
				ratio::Ratio::one(),
				ratio::Ratio::zero(),
				ratio::Ratio::zero(),
			) );
		let duration = gen.generate_value( &format!( "out.{}.duration", ch ) )?
			.unwrap_or( generator::ValueIr::Symbol( "note.len".into() ) );
		let pitch = gen.generate_value( &format!( "out.{}.pitch", ch ) )?
			.unwrap_or( generator::ValueIr::Value(
				ratio::Ratio::zero(),
				ratio::Ratio::one(),
				ratio::Ratio::zero(),
				ratio::Ratio::zero(),
			) );
		let mut ccs = Vec::new();
		for cc in 0 .. 128 {
			if let Some( ir ) = gen.generate_value( &format!( "out.{}.cc{}", ch, cc ) )? {
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

	let tempo = gen.generate_value( "out.tempo" )?
		.unwrap_or( generator::ValueIr::Value(
			ratio::Ratio::zero(),
			ratio::Ratio::one(),
			ratio::Ratio::one(),
			ratio::Ratio::one(),
		) );

	let len = channels.iter()
		.flat_map( |&(_, ref v)| v.score.iter() )
		.map( |v| v.t1 )
		.max()
		.unwrap_or( ratio::Ratio::zero() );

	let evaluator = generator::Evaluator::new_with_random( rng );
	let bgn = match gen.generate_value( "out.begin" )? {
		Some( ir ) => (evaluator.eval( &ir, ratio::Ratio::zero() ) * TICK as f64).round() as i64,
		None       => 0,
	};
	let end = match gen.generate_value( "out.end" )? {
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

pub fn assemble( rng: &random::Generator, src: &Assembly ) -> Result<Vec<midi::Event>, misc::Error> {
	let bgn = (src.bgn * TICK).round();
	let end = (src.end * TICK).round();
	let mut migen = midi::Generator::new( rng, bgn, end, TICK );
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
