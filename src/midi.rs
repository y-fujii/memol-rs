// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD license.
use std::*;
use irgen;


#[derive(Debug)]
pub struct Event {
	pub time: i32,
	pub prio: i32,
	pub len: i32,
	pub msg: [u8; 4],
}

impl Event {
	fn new( time: i32, prio: i32, msg: &[u8] ) -> Event {
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
	base: i32,
	dst: Vec<Event>,
}

impl Generator {
	pub fn new( base: i32 ) -> Generator {
		Generator{ base: base, dst: Vec::new() }
	}

	pub fn add_score( &mut self, ch: i32, src: &Vec<irgen::FlatNote> ) {
		let vel = 79;
		for f in src.iter() {
			self.dst.push( Event::new(
				(f.bgn * self.base).to_int(), 1,
				&[ (0x90 + ch) as u8, f.nnum as u8, vel ],
			) );
			self.dst.push( Event::new(
				(f.end * self.base).to_int(), 0,
				&[ (0x80 + ch) as u8, f.nnum as u8, 0 ],
			) );
		}
	}

	pub fn generate( &mut self ) -> Vec<Event> {
		let mut tmp = Vec::new();
		mem::swap( &mut tmp, &mut self.dst );
		tmp.sort_by_key( |e| (e.time, e.prio) );
		tmp
	}
}
