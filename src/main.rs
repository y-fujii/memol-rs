// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
#![feature( untagged_unions )]

extern crate lalrpop_util;
extern crate getopts;
#[allow( dead_code )]
#[allow( non_upper_case_globals )]
#[allow( non_camel_case_types )]
#[allow( improper_ctypes )]
mod cext;
mod misc;
mod ratio;
mod ast;
mod parser;
mod irgen;
mod midi;
mod player;
mod notify;
use std::*;
use std::io::prelude::*;
use std::str::FromStr;


fn compile( src: &str, dump: bool ) -> Result<Vec<midi::Event>, misc::Error> {
	let now = time::SystemTime::now();
	let tree = match parser::parse_definition( src ) {
		Err( e ) => return misc::error( &format!( "{:?}", e ) ),
		Ok ( v ) => v,
	};

	let irgen = irgen::Generator::new( &tree );
	let mut migen = midi::Generator::new();
	for ch in 0 .. 16 {
		if let Some( ir ) = irgen.generate( &format!( "out.{}", ch ) )? {
			if dump {
				println!( "channel {}:", ch );
				for f in ir.iter() {
					if let Some( nnum ) = f.nnum {
						println!( "\t{}/{} .. {}/{} : {}", f.bgn.y, f.bgn.x, f.end.y, f.end.x, nnum );
					}
				}
			}
			migen = migen.add_score( ch, &ir );
		}
	}
	if dump {
		let elapsed = now.elapsed().unwrap();
		println!( "parsing & generation: {} ms.", elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1000000 );
	}
	Ok( migen.generate() )
}

fn main() {
	|| -> Result<(), Box<error::Error>> {
		let mut opts = getopts::Options::new();
		opts.optmulti( "c", "connect", "", "" );
		opts.optflag( "d", "dump", "" );
		opts.optopt( "s", "seek", "", "" );
		let args = opts.parse( env::args().skip( 1 ) )?;
		let seek = match args.opt_str( "s" ) {
			Some( ref v ) => Some( i64::from_str( v )? ),
			None          => None,
		};
		if args.free.len() != 1 {
			return misc::error( "" );
		}

		let mut player = player::Player::new( "memol" )?;
		for port in args.opt_strs( "c" ) {
			player.connect( &port )?;
		}

		loop {
			let mut buf = String::new();
			fs::File::open( &args.free[0] )?.read_to_string( &mut buf )?;

			match compile( &buf, args.opt_present( "d" ) ) {
				Err( e ) => {
					println!( "error: {}", e );
				},
				Ok( ev ) => {
					player.set_data( ev );
					if let Some( t ) = seek {
						player.seek( ratio::Ratio::new( t, 1 ) )?;
						player.play()?;
					}
				},
			}

			notify::notify_wait( &args.free[0] )?;
		}
	}().unwrap();
}
