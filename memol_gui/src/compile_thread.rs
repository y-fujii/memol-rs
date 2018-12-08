// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use memol::*;
use memol_cli::notify;


pub enum Message {
	File( path::PathBuf ),
	Refresh,
	Exit,
}

pub struct CompileThread {
	tx: sync::mpsc::Sender<Message>,
	rx: sync::mpsc::Receiver<Message>,
	on_success: Box<dyn FnMut( path::PathBuf, Assembly, Vec<midi::Event> ) + marker::Send>,
	on_failure: Box<dyn FnMut( String ) + marker::Send>,
}

impl CompileThread {
	pub fn new() -> Self {
		let (tx, rx) = sync::mpsc::channel();
		CompileThread{
			tx: tx,
			rx: rx,
			on_success: Box::new( |_, _, _| () ),
			on_failure: Box::new( |_| () ),
		}
	}

	pub fn on_success<T: 'static + FnMut( path::PathBuf, Assembly, Vec<midi::Event> ) + marker::Send>( &mut self, f: T ) {
		self.on_success = Box::new( f );
	}

	pub fn on_failure<T: 'static + FnMut( String ) + marker::Send>( &mut self, f: T ) {
		self.on_failure = Box::new( f );
	}

	pub fn create_sender( &self ) -> sync::mpsc::Sender<Message> {
		self.tx.clone()
	}

	pub fn spawn( mut self ) -> thread::JoinHandle<()> {
		thread::spawn( move || {
			let mut path = path::PathBuf::new();
			let mut modified = time::UNIX_EPOCH;
			loop {
				match notify::wait_file_or_channel( &path, &self.rx, modified ) {
					notify::WaitResult::File( v ) => {
						modified = v;
					},
					notify::WaitResult::Channel( Message::File( v ) ) => {
						path = v;
						modified = time::UNIX_EPOCH;
						continue;
					},
					notify::WaitResult::Channel( Message::Refresh ) => (),
					notify::WaitResult::Channel( Message::Exit ) => break,
					notify::WaitResult::Disconnect => break,
				}
				if path == path::PathBuf::new() {
					continue;
				}

				let rng = random::Generator::new();
				let asm = match compile( &rng, &path ) {
					Ok ( v ) => v,
					Err( e ) => {
						(self.on_failure)( format!( "{}", e ) );
						continue;
					},
				};
				let evs = match assemble( &rng, &asm ) {
					Ok ( v ) => v,
					Err( e ) => {
						(self.on_failure)( format!( "{}", e ) );
						continue;
					},
				};
				(self.on_success)( path.clone(), asm, evs );
			}
		} )
	}
}
