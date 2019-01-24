// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use memol_cli::*;


fn compile( path: &path::Path, verbose: bool ) -> Option<Vec<memol::midi::Event>> {
	let timer = time::SystemTime::now();
	let rng = memol::random::Generator::new();
	let result = memol::compile( &rng, &path ).and_then( |e| memol::assemble( &rng, &e ) );
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
	let f = || -> Result<(), Box<dyn error::Error>> {
		// parse command line.
		let mut opts = getopts::Options::new();
		opts.optflag ( "v", "verbose", "" );
		opts.optflag ( "b", "batch",   "Generate a MIDI file." );
		opts.optmulti( "c", "connect", "Connect to a JACK port.", "PORT" );
		opts.optopt  ( "a", "address", "WebSocket address.", "ADDR:PORT" );
		let args = opts.parse( env::args().skip( 1 ) )?;
		if args.free.len() != 1 {
			print!( "{}", opts.usage( "Usage: memol_cli [options] FILE" ) );
			return Ok( () );
		}
		let path = path::PathBuf::from( &args.free[0] );

		// generate MIDI file.
		if args.opt_present( "b" ) {
			if let Some( events ) = compile( &path, args.opt_present( "v" ) ) {
				let smf = memol::smf::generate_smf( &events, 480 );
				fs::write( path.with_extension( "mid" ), smf )?;
			}
			return Ok( () );
		}

		// initialize IPC.
		let addr = args.opt_str( "a" ).unwrap_or( "127.0.0.1:27182".into() );
		let bus = ipc::Bus::new();
		let sender: ipc::Sender<ipc::Message> = bus.create_sender();
		thread::spawn( move || {
			if let Err( err ) = bus.listen( addr, |_| () ) {
				eprintln!( "IPC: {}", err );
			}
		} );

		// initialize JACK.
		let player: Box<dyn player::Player> = match player_jack::Player::new( "memol" ) {
			Ok ( v ) => v,
			Err( _ ) => player::DummyPlayer::new(),
		};
		for port in args.opt_strs( "c" ) {
			player.connect_to( &port )?;
		}

		// main loop.
		loop {
			if let Some( events ) = compile( &path, args.opt_present( "v" ) ) {
				let bgn = match events.get( 0 ) {
					Some( ev ) => ev.time.max( 0.0 ),
					None       => 0.0,
				};
				player.set_data( events.clone() );
				player.seek( bgn )?;
				player.play()?;
				sender.send( &ipc::Message::Success{
					events: events.into_iter().map( |e| e.into() ).collect()
				} )?;
			}

			notify::wait_file( &args.free[0] )?;
		}
	};
	if let Err( e ) = f() {
		eprintln!( "{}", e );
	}
}
