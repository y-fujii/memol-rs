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
		for f in score.notes.iter() {
			if let Some( nnum ) = f.nnum {
				// XXX
				let vel = (vels.get_value( f.t0 ) * ratio::Ratio::new( 127, 8 )).round() as u8;
				self.events.push( Event::new( f.t0,  1, &[ (0x90 + ch) as u8, nnum as u8, vel ] ) );
				self.events.push( Event::new( f.t1, -1, &[ (0x80 + ch) as u8, nnum as u8, vel ] ) );
			}
		}
		self
	}

	pub fn add_cc( mut self, ch: i32, cc: i32, value: &valuegen::Ir ) -> Generator {
		for f in value.values.iter() {
			// XXX
			let v0 = (f.v0 * ratio::Ratio::new( 127, 8 )).floor();
			let v1 = (f.v1 * ratio::Ratio::new( 127, 8 )).ceil();
			self.events.push( Event::new( f.t0, 0, &[ (0xb0 + ch) as u8, cc as u8, v0 as u8 ] ) );
			for v in v0 + 1 .. v1 {
				let t = f.t0 + (f.t1 - f.t0) * (v - f.v0) / (f.v1 - f.v0);
				self.events.push( Event::new( t, 0, &[ (0xb0 + ch) as u8, cc as u8, v as u8 ] ) );
			}
		}
		self
	}

	pub fn generate( mut self ) -> Vec<Event> {
		self.events.sort_by_key( |e| (e.time, e.prio) );
		self.events
	}
}
