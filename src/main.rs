// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
#![feature( untagged_unions )]

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


fn compile( src: &str ) -> Result<Vec<midi::Event>, misc::Error> {
	let tree = match parser::parse_definition( src ) {
		Err( e ) => return misc::Error::new( &format!( "{:?}", e ) ),
		Ok ( v ) => v,
	};
	let ir = irgen::Generator::new( &tree ).generate( "root" )?;
	Ok( midi::Generator::new().add_score( 0, &ir ).generate() )
}

fn main() {
	let args: Vec<String> = env::args().collect();

	|| -> Result<(), Box<error::Error>> {
		let mut player = player::Player::new( "memol", &args[2] )?;
		loop {
			let mut buf = String::new();
			fs::File::open( &args[1] )?.read_to_string( &mut buf )?;

			match compile( &buf ) {
				Err( e ) => {
					println!( "error: {:}", e );
				},
				Ok( ev ) => {
					player.set_data( ev );
					player.seek( ratio::Ratio::new( 0, 1 ) )?;
					player.play()?;
				},
			}

			notify::notify_wait( &args[1] )?;
		}
	}().unwrap();
}
