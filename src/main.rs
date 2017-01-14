// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
extern crate getopts;
extern crate memol;
use std::*;
use std::io::prelude::*;
use std::str::FromStr;
use memol::*;


fn main() {
	|| -> Result<(), Box<error::Error>> {
		let mut opts = getopts::Options::new();
		opts.optmulti( "c", "connect", "", "" );
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

			match compile( &buf ) {
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
