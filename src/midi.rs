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
	cc_span: i64,
}

impl Generator {
	pub fn new() -> Generator {
		Generator{
			events: Vec::new(),
			cc_span: 480,
		}
	}

	pub fn add_score( mut self, ch: i32, score: &scoregen::Ir, vels: &valuegen::Ir ) -> Generator {
		for f in score.notes.iter() {
			if let Some( nnum ) = f.nnum {
				let vel = (vels.value( f.t0 ) * 127).round();
				let vel = cmp::min( cmp::max( vel, 0 ), 127 );
				self.events.push( Event::new( f.t0,  1, &[ (0x90 + ch) as u8, nnum as u8, vel as u8 ] ) );
				self.events.push( Event::new( f.t1, -1, &[ (0x80 + ch) as u8, nnum as u8, vel as u8 ] ) );
			}
		}
		self
	}

	pub fn add_cc( mut self, ch: i32, cc: i32, ir: &valuegen::Ir ) -> Generator {
		let mut prev_v = 255;
		for i in 0 .. (ir.end() * self.cc_span).ceil() {
			let t = ratio::Ratio::new( i, self.cc_span );
			let v = (ir.value( t ) * 127).round();
			let v = cmp::min( cmp::max( v, 0 ), 127 );
			if v != prev_v {
				self.events.push( Event::new( t, 0, &[ (0xb0 + ch) as u8, cc as u8, v as u8 ] ) );
				prev_v = v;
			}
		}
		self
	}

	pub fn generate( mut self ) -> Vec<Event> {
		self.events.sort_by_key( |e| (e.time, e.prio) );
		self.events
	}
}
