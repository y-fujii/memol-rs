// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use ratio;
use irgen;


#[derive(Debug)]
pub struct Event {
	pub time: ratio::Ratio,
	pub prio: i32,
	pub len: i32,
	pub msg: [u8; 4],
}

impl Event {
	fn new( time: ratio::Ratio, prio: i32, msg: &[u8] ) -> Event {
		let mut this = Event{
			time: time,
			prio: prio,
			len: msg.len() as i32,
			msg: [0; 4]
		};
		this.msg[..msg.len()].copy_from_slice( msg );
		this
	}
}

#[derive(Debug)]
pub struct Generator {
	dst: Vec<Event>,
}

impl Generator {
	pub fn new() -> Generator {
		Generator{ dst: Vec::new() }
	}

	pub fn add_score( mut self, ch: i32, src: &Vec<irgen::FlatNote> ) -> Generator {
		let vel = 79;
		for f in src.iter() {
			if let Some( nnum ) = f.nnum {
				self.dst.push( Event::new( f.bgn * 2, 1, &[ (0x90 + ch) as u8, nnum as u8, vel ] ) );
				self.dst.push( Event::new( f.end * 2, 0, &[ (0x80 + ch) as u8, nnum as u8, vel ] ) );
			}
		}
		self
	}

	pub fn generate( &mut self ) -> Vec<Event> {
		let mut tmp = mem::replace( &mut self.dst, Vec::new() );
		tmp.sort_by_key( |e| (e.time, e.prio) );
		tmp
	}
}
