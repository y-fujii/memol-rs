// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
extern crate getopts;
extern crate memol;
use std::*;
use memol::*;


fn main() {
	let f = || -> Result<(), Box<error::Error>> {
		let mut opts = getopts::Options::new();
		opts.optmulti( "c", "", "", "" );
		let args = opts.parse( env::args().skip( 1 ) )?;
		if args.free.len() != 1 {
			println!( "Usage: memol (-c JACK_PORT)* FILE" );
			return Ok( () );
		}

		let player = player::Player::new( "memol" )?;
		for port in args.opt_strs( "c" ) {
			player.connect( &port )?;
		}

		loop {
			let path = path::PathBuf::from( &args.free[0] );
			let timer = time::SystemTime::now();
			let result = compile( &path ).and_then( |e| assemble( &e ) );
			let elapsed = timer.elapsed()?;
			eprintln!( "compile time: {} ms", elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1000000 );
			eprintln!( " event count: {}", result.as_ref().map( |evs| evs.len() ).unwrap_or( 0 ) );
			match result {
				Err( e ) => {
					println!( "{}", e );
				},
				Ok( events ) => {
					let bgn = match events.get( 0 ) {
						Some( ev ) => ev.time.max( 0.0 ),
						None       => 0.0,
					};
					player.set_data( events );
					player.seek( bgn )?;
					player.play()?;
				},
			}

			notify::wait_file( &args.free[0] )?;
		}
	};
	if let Err( e ) = f() {
		println!( "error: {}", e );
	}
}
