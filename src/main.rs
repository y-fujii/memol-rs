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

			let timer = time::SystemTime::now();
			let result = compile( &buf );
			let elapsed = timer.elapsed()?;
			println!( "compile time: {} ms", elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1000000 );
			println!( " event count: {}", result.as_ref().map( |&(ref evs, _)| evs.len() ).unwrap_or( 0 ) );
			match result {
				Err( e ) => {
					let (row, col) = misc::text_row_col( &buf[0 .. e.loc] );
					println!( "error at ({}, {}): {}", row, col, e.msg );
				},
				Ok( (events, range) ) => {
					match range {
						Some( (bgn, end) ) => {
							player.set_data_with_range( events, bgn, end );
							player.seek( bgn )?;
							player.play()?;
						},
						None => {
							player.set_data( events );
						}
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
