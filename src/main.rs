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


fn main() {
	|| -> Result<(), Box<error::Error>> {
		let src_path = "test.mol";

		{
			let mut buf = String::new();
			fs::File::open( src_path )?.read_to_string( &mut buf )?;

			let tree = parser::parse_definition( &buf ).unwrap();
			println!( "{:?}", tree );

			let irgen = irgen::Generator::new( &tree );
			let ir = irgen.generate( "root" )?;
			let mut migen = midi::Generator::new( 48000 );
			migen.add_score( 0, &ir );
			let dst = migen.generate();
			println!( "{:?}", dst );

			let mut player = player::Player::new( "memol" )?;
			player.set_data( dst );
			player.activate()?;

			notify::notify_wait( src_path )?;
			Ok( () )
		}

		/*
		loop {
			let mut buf = String::new();
			fs::File::open( src_path )?.read_to_string( &mut buf )?;

			let tree = parser::parse_definition( &buf );
			println!( "{:?}", tree );

			if let Ok( tree ) = tree {
				let irgen = irgen::Generator::new( &tree );
				if let Ok( flats ) = irgen.generate( "root" ) {
					for f in flats.iter() {
						println!( "{}/{} .. {}/{} : {}", f.bgn.y, f.bgn.x, f.end.y, f.end.x, f.nnum );
					}
					let mut migen = midi::Generator::new( 48000 );
					migen.add_score( 0, &flats );
					let dst = migen.generate();
					println!( "{:?}", dst );
					player.set_data( dst );
				}
			}

			notify::notify_wait( src_path )?;
		}
		*/
	}().unwrap();
}
