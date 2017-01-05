// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
extern crate regex;
extern crate lalrpop_util;
pub mod misc;
pub mod ratio;
pub mod ast;
pub mod parser;
pub mod irgen;
pub mod midi;
pub mod jack;
pub mod player;
pub mod notify;


pub fn compile( src: &str ) -> Result<Vec<midi::Event>, misc::Error> {
	// XXX
	let src = regex::Regex::new( r"(?s:/\*.*?\*/)" ).unwrap().replace_all( src, "" );
	let tree = match parser::parse_definition( &src ) {
		Err( e ) => return misc::error( &format!( "{:?}", e ) ),
		Ok ( v ) => v,
	};

	let irgen = irgen::Generator::new( &tree );
	let mut migen = midi::Generator::new();
	for ch in 0 .. 16 {
		if let Some( ir ) = irgen.generate( &format!( "out.{}", ch ) )? {
			migen = migen.add_score( ch, &ir );
		}
	}
	Ok( migen.generate() )
}
