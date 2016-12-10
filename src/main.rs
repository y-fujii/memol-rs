// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
#![feature( untagged_unions )]

#[allow( dead_code )]
#[allow( non_upper_case_globals )]
#[allow( non_camel_case_types )]
#[allow( improper_ctypes )]
mod cext;
mod ratio;
mod ast;
mod parser;
mod generator;
mod player;
mod notify;
use std::*;
use std::io::prelude::*;


fn main() {
	|| -> Result<(), Box<error::Error>> {
		let src_path = "test.mol";
		let mut _jack = player::Player::new( "memol" )?;

		loop {
			let mut buf = String::new();
			fs::File::open( src_path )?.read_to_string( &mut buf )?;

			let tree = parser::parse_definition( &buf );
			println!( "{:?}", tree );

			if let Ok( tree ) = tree {
				let genr = generator::Generator::new( &tree );
				if let Some( flats ) = genr.generate( "root" ) {
					for f in flats.iter() {
						println!( "{}/{} .. {}/{} : {}", f.bgn.y, f.bgn.x, f.end.y, f.end.x, f.nnum );
					}
				}
			}

			notify::notify_wait( src_path )?;
		}
	}().unwrap();
}
