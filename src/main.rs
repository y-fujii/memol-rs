// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
extern crate getopts;
extern crate memol;
use std::*;
use std::io::prelude::*;
use memol::*;


fn main() {
	let f = || -> Result<(), Box<error::Error>> {
		let mut opts = getopts::Options::new();
		opts.optmulti( "c", "connect", "", "" );
		let args = opts.parse( env::args().skip( 1 ) )?;
		if args.free.len() != 1 {
			return Err( getopts::Fail::UnexpectedArgument( String::new() ).into() );
		}

		let mut player = player::Player::new( "memol" )?;
		for port in args.opt_strs( "c" ) {
			player.connect( &port )?;
		}

		loop {
			let mut buf = String::new();
			fs::File::open( &args.free[0] )?.read_to_string( &mut buf )?;

			match compile( &buf ) {
				Err( e ) => {
					let (row, col) = misc::text_row_col( &buf[0 .. e.loc] );
					println!( "error at ({}, {}): {}", row, col, e.msg );
				},
				Ok( (events, marks) ) => {
					if marks.len() >= 2 {
						player.set_data_with_range( events, marks[0], marks[1] );
						player.seek( marks[0] )?;
						player.play()?;
					}
					else {
						player.set_data( events );
					}
				},
			}

			notify::notify_wait( &args.free[0] )?;
		}
	};
	if let Err( e ) = f() {
		println!( "error: {}", e.description() );
	}
}
