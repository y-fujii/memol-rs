// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use rand;
use misc;
use ratio::Ratio;
use scoregen;
use valuegen;


#[derive(Debug)]
pub struct Event {
	pub time: f64,
	pub prio: i16,
	pub len: i16,
	pub msg: [u8; 4],
}

impl Event {
	fn new( time: f64, prio: i16, msg: &[u8] ) -> Event {
		let mut this = Event{
			time: time,
			prio: prio,
			len: msg.len() as i16,
			msg: [0; 4]
		};
		this.msg[..msg.len()].copy_from_slice( msg );
		this
	}
}

#[derive(Debug)]
pub struct Generator<'a, T: 'a + rand::Rng> {
	rng: &'a mut T,
	events: Vec<Event>,
	timeline: Vec<f64>,
	bgn: i64,
	end: i64,
	tick: i64,
}

impl<'a, T: 'a + rand::Rng> Generator<'a, T> {
	pub fn new( rng: &'a mut T, bgn: i64, end: i64, tick: i64 ) -> Self {
		Generator{
			rng: rng,
			events: Vec::new(),
			timeline: Vec::new(),
			bgn: bgn,
			end: end,
			tick: tick,
		}
	}

	pub fn add_score( &mut self, ch: usize, ir_score: &scoregen::Ir, ir_vel: &valuegen::Ir, ir_ofs: &valuegen::Ir, ir_dur: &valuegen::Ir ) {
		let note_len = cell::Cell::new( 0.0 );
		let mut evaluator = valuegen::Evaluator::new_with_random( self.rng );
		evaluator.add_symbol( "note_len".into(), |_| note_len.get() );
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

			note_len.set( (f.t1 - f.t0).to_float() );
			let dt = evaluator.eval( ir_dur, f.t0 );
			let d0 = *offset.entry( (f.t0, nnum) ).or_insert_with( || evaluator.eval( ir_ofs, f.t0 ) );
			let d1 = *offset.entry( (f.t1, nnum) ).or_insert_with( || evaluator.eval( ir_ofs, f.t1 ) );
			let t0 = f.t0.to_float() + d0;
			let t1 = if dt == note_len.get() {
				// avoid event order inversion due to FP errors.
				f.t1.to_float() + d1
			}
			else {
				let a = dt / note_len.get();
				(1.0 - a) * (f.t0.to_float() + d0) + a * (f.t1.to_float() + d1)
			};
			if t0 >= t1 {
				continue;
			}
			let vel = (evaluator.eval( ir_vel, f.t0 ) * 127.0).round().max( 0.0 ).min( 127.0 );
			self.events.push( Event::new( t0,  1, &[ (0x90 + ch) as u8, nnum as u8, vel as u8 ] ) );
			self.events.push( Event::new( t1, -1, &[ (0x80 + ch) as u8, nnum as u8, vel as u8 ] ) );
		}
	}

	pub fn add_cc( &mut self, ch: usize, cc: usize, ir: &valuegen::Ir ) {
		let mut evaluator = valuegen::Evaluator::new_with_random( self.rng );
		let mut prev_v = 255;
		for i in self.bgn .. self.end {
			let t = Ratio::new( i, self.tick );
			let v = (evaluator.eval( ir, t ) * 127.0).round().max( 0.0 ).min( 127.0 ) as u8;
			if v != prev_v {
				self.events.push( Event::new( t.to_float(), 0, &[ (0xb0 + ch) as u8, cc as u8, v ] ) );
				prev_v = v;
			}
		}
	}

	pub fn add_tempo( &mut self, ir: &valuegen::Ir ) {
		let mut evaluator = valuegen::Evaluator::new_with_random( self.rng );
		debug_assert!( self.timeline.len() == 0 );
		let mut s = 0.0;
		for i in 0 .. self.end + 1 {
			self.timeline.push( s );
			s += 1.0 / (self.tick as f64 * evaluator.eval( ir, Ratio::new( i, self.tick ) ));
		}
		self.timeline.push( s );
	}

	pub fn generate( mut self ) -> Result<Vec<Event>, misc::Error> {
		self.events.sort_by( |x, y| (x.time, x.prio).partial_cmp( &(y.time, y.prio) ).unwrap() );
		if self.timeline.len() > 0 {
			for ev in self.events.iter_mut() {
				let i = (ev.time * self.tick as f64).floor() as usize;
				let i = cmp::min( cmp::max( i, 0 ), self.timeline.len() - 2 );
				let f0 = self.timeline[i + 0];
				let f1 = self.timeline[i + 1];
				let a = ev.time * self.tick as f64 - i as f64;
				ev.time = (1.0 - a) * f0 + a * f1;
			}
		}
		Ok( self.events )
	}
}
