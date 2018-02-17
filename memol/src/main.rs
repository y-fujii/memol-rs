// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
extern crate getopts;
extern crate memol;
use std::*;


fn compile( path: &path::Path, verbose: bool ) -> Option<Vec<memol::midi::Event>> {
	let timer = time::SystemTime::now();
	let result = memol::compile( &path ).and_then( |e| memol::assemble( &e ) );
	let elapsed = timer.elapsed().unwrap();
	if verbose {
		eprintln!( "compile time: {} ms", elapsed.as_secs() * 1000 + elapsed.subsec_nanos() as u64 / 1000000 );
		eprintln!( " event count: {}", result.as_ref().map( |evs| evs.len() ).unwrap_or( 0 ) );
	}
	match result {
		Err( e ) => {
			println!( "{}", e );
			None
		},
		Ok( v ) => Some( v ),
	}
}

fn main() {
	let f = || -> Result<(), Box<error::Error>> {
		let mut opts = getopts::Options::new();
		opts.optflag ( "v", "verbose", "" );
		opts.optflag ( "b", "batch",   "Generate a SMF file." );
		opts.optmulti( "c", "connect", "Connect to a JACK port.", "" );
		let args = opts.parse( env::args().skip( 1 ) )?;
		if args.free.len() != 1 {
			print!( "{}", opts.usage( "Usage: memol [options] FILE" ) );
			return Ok( () );
		}

		let path = path::PathBuf::from( &args.free[0] );

		if args.opt_present( "b" ) {
			if let Some( events ) = compile( &path, args.opt_present( "v" ) ) {
				let smf = path.with_extension( "mid" );
				let mut buf = io::BufWriter::new( fs::File::create( smf )? );
				memol::smf::write_smf( &mut buf, &events, 480 )?;
			}
			return Ok( () );
		}

		let player = memol::player::Player::new( "memol" )?;
		for port in args.opt_strs( "c" ) {
			player.connect( &port )?;
		}

		loop {
			if let Some( events ) = compile( &path, args.opt_present( "v" ) ) {
				let bgn = match events.get( 0 ) {
					Some( ev ) => ev.time.max( 0.0 ),
					None       => 0.0,
				};
				player.set_data( events );
				player.seek( bgn )?;
				player.play()?;
			}

			memol::notify::wait_file( &args.free[0] )?;
		}
	};
	if let Err( e ) = f() {
		println!( "{}", e );
	}
}
