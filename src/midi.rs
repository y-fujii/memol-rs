// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use ratio::Ratio;
use scoregen;
use valuegen;


#[derive(Debug)]
pub struct Event {
	pub time: f64,
	pub prio: i32,
	pub len: i32,
	pub msg: [u8; 4],
}

impl Event {
	fn new( time: f64, prio: i32, msg: &[u8] ) -> Event {
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
	timeline: Vec<f64>,
	bgn: i64,
	end: i64,
	tick: i64,
}

impl Generator {
	pub fn new( bgn: i64, end: i64, tick: i64 ) -> Generator {
		Generator{
			events: Vec::new(),
			timeline: Vec::new(),
			bgn: bgn,
			end: end,
			tick: tick,
		}
	}

	pub fn add_score( &mut self, ch: usize, ir_score: &scoregen::Ir, ir_vel: &valuegen::Ir, ir_ofs: &valuegen::Ir ) {
		let mut offset = collections::HashMap::new();
		for f in ir_score.notes.iter() {
			let nnum = match f.nnum {
				Some( v ) => v,
				None      => continue,
			};
			// accepts note off messages at end.
			if f.t0 < Ratio::new( self.bgn, self.tick ) || Ratio::new( self.end, self.tick ) < f.t1 {
				continue;
			}
			let t0 = f.t0.to_float() + *offset.entry( (f.t0, nnum) ).or_insert_with( || ir_ofs.value( f.t0 ) );
			let t1 = f.t1.to_float() + *offset.entry( (f.t1, nnum) ).or_insert_with( || ir_ofs.value( f.t1 ) );
			if t0 >= t1 {
				continue;
			}
			let vel = (ir_vel.value( f.t0 ) * 127.0).round().max( 0.0 ).min( 127.0 );
			self.events.push( Event::new( t0,  1, &[ (0x90 + ch) as u8, nnum as u8, vel as u8 ] ) );
			self.events.push( Event::new( t1, -1, &[ (0x80 + ch) as u8, nnum as u8, vel as u8 ] ) );
		}
	}

	pub fn add_cc( &mut self, ch: usize, cc: usize, ir: &valuegen::Ir ) {
		let mut prev_v = 255;
		for i in self.bgn .. self.end {
			let t = Ratio::new( i, self.tick );
			let v = (ir.value( t ) * 127.0).round().max( 0.0 ).min( 127.0 ) as u8;
			if v != prev_v {
				self.events.push( Event::new( t.to_float(), 0, &[ (0xb0 + ch) as u8, cc as u8, v ] ) );
				prev_v = v;
			}
		}
	}

	pub fn add_tempo( &mut self, ir: &valuegen::Ir ) {
		assert!( self.timeline.len() == 0 );
		let mut s = 0.0;
		for i in 0 .. self.end + 2 {
			self.timeline.push( s );
			s += 1.0 / (self.tick as f64 * ir.value( Ratio::new( i, self.tick ) ));
		}
	}

	pub fn generate( mut self ) -> Result<Vec<Event>, misc::Error> {
		self.events.sort_by( |x, y| (x.time, x.prio).partial_cmp( &(y.time, y.prio) ).unwrap() );
		if self.timeline.len() > 0 {
			for ev in self.events.iter_mut() {
				let i = (ev.time * self.tick as f64).floor() as usize;
				if i + 1 >= self.timeline.len() {
					return misc::error( 0, "tempo track is too short." );
				}
				let f0 = self.timeline[i + 0];
				let f1 = self.timeline[i + 1];
				let a = (ev.time * self.tick as f64).fract();
				ev.time = (1.0 - a) * f0 + a * f1;
			}
		}
		Ok( self.events )
	}
}
