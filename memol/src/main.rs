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

		let player = player::Player::new( "memol" )?;
		for port in args.opt_strs( "c" ) {
			player.connect( &port )?;
		}

		loop {
			let mut buf = String::new();
			fs::File::open( &args.free[0] )?.read_to_string( &mut buf )?;

			let timer = time::SystemTime::now();
			let result = compile( &buf ).and_then( |e| assemble( &e ) );
			let elapsed = timer.elapsed()?;
			println!( "compile time: {} ms", elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1000000 );
			println!( " event count: {}", result.as_ref().map( |evs| evs.len() ).unwrap_or( 0 ) );
			match result {
				Err( e ) => {
					let (row, col) = misc::text_row_col( &buf[0 .. e.loc] );
					println!( "error at ({}, {}): {}", row, col, e.msg );
				},
				Ok( events ) => {
					let bgn = match events.get( 0 ) {
						Some( ev ) => ev.time,
						None       => 0.0,
					};
					player.set_data( events );
					player.seek( bgn )?;
					player.play()?;
				},
			}

			notify::notify_wait( &args.free[0] )?;
		}
	};
	if let Err( e ) = f() {
		println!( "error: {}", e.description() );
	}
}
