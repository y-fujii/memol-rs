// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use ratio;
use scoregen;
use valuegen;


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
	events: Vec<Event>,
}

impl Generator {
	pub fn new() -> Generator {
		Generator{
			events: Vec::new(),
		}
	}

	pub fn add_score( mut self, ch: i32, score: &scoregen::Ir, vels: &valuegen::Ir ) -> Generator {
		for n in score.notes.iter() {
			if let Some( nnum ) = n.nnum {
				// XXX
				let vel = (vels.get_value( n.bgn ) * ratio::Ratio::new( 127, 9 )).to_int() as u8;
				self.events.push( Event::new( n.bgn * 2, 1, &[ (0x90 + ch) as u8, nnum as u8, vel ] ) );
				self.events.push( Event::new( n.end * 2, 0, &[ (0x80 + ch) as u8, nnum as u8, vel ] ) );
			}
		}
		self
	}

	/*
	pub fn add_cc( mut self, ch: i32, cc: i32, value: &valuegen::Ir ) -> Generator {
		self
	}
	*/

	pub fn generate( mut self ) -> Vec<Event> {
		self.events.sort_by_key( |e| (e.time, e.prio) );
		self.events
	}
}
