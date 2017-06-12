// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use rand;
use misc;
use ratio;
use ast;


#[derive(Debug)]
pub enum Ir {
	Value( ratio::Ratio, ratio::Ratio, ratio::Ratio, ratio::Ratio ),
	Sequence( Vec<(Ir, ratio::Ratio)> ),
	BinaryOp( Box<Ir>, Box<Ir>, ast::BinaryOp ),
	Gaussian,
}

#[derive(Debug)]
struct Span {
	t0: ratio::Ratio,
	t1: ratio::Ratio,
}

#[derive(Debug)]
struct State {
}

impl Ir {
	pub fn value( &self, t: ratio::Ratio ) -> f64 {
		match *self {
			Ir::Value( t0, t1, v0, v1 ) => {
				let t = cmp::min( cmp::max( t, t0 ), t1 );
				let v = v0 + (v1 - v0) * (t - t0) / (t1 - t0);
				v.to_float()
			},
			Ir::Sequence( ref irs ) => {
				let i = misc::bsearch_boundary( &irs, |&(_, t0)| t0 <= t );
				irs[i - 1].0.value( t )
			},
			Ir::BinaryOp( ref ir_lhs, ref ir_rhs, op ) => {
				let lhs = ir_lhs.value( t );
				let rhs = ir_rhs.value( t );
				match op {
					ast::BinaryOp::Add => lhs + rhs,
					ast::BinaryOp::Sub => lhs - rhs,
					ast::BinaryOp::Mul => lhs * rhs,
					ast::BinaryOp::Div => lhs / rhs,
				}
			},
			Ir::Gaussian => {
				let rand::distributions::normal::StandardNormal( x ) = rand::random();
				x
			},
		}
	}
}

#[derive(Debug)]
pub struct Generator<'a> {
	defs: &'a ast::Definition<'a>,
}

impl<'a> Generator<'a> {
	pub fn new( defs: &'a ast::Definition<'a> ) -> Generator<'a> {
		Generator{ defs: defs }
	}

	pub fn generate( &self, key: &str ) -> Result<Option<Ir>, misc::Error> {
		let span = Span{
			t0: ratio::Ratio::zero(),
			t1: ratio::Ratio::one(),
		};
		let s = match self.defs.values.get( key ) {
			Some( v ) => v,
			None      => return Ok( None ),
		};
		let (ir, _) = self.generate_value_track( s, &span )?;
		Ok( Some( ir ) )
	}

	fn generate_value_track( &self, track: &ast::Ast<ast::ValueTrack>, span: &Span ) -> Result<(Ir, ratio::Ratio), misc::Error> {
		let dst = match track.ast {
			ast::ValueTrack::ValueTrack( ref vs ) => {
				let mut irs = Vec::new();
				let mut state = State{};
				for (i, v) in vs.iter().enumerate() {
					let span = Span{
						t0: span.t0 + (span.t1 - span.t0) * i as i64,
						t1: span.t1 + (span.t1 - span.t0) * i as i64,
						.. *span
					};
					self.generate_value( v, &span, &mut state, &mut irs )?;
				}
				let t1 = span.t0 + (span.t1 - span.t0) * vs.len() as i64;
				(Ir::Sequence( irs ), t1)
			},
			ast::ValueTrack::Symbol( ref key ) => {
				let s = match self.defs.values.get( key ) {
					Some( v ) => v,
					None      => return misc::error( track.bgn, "undefined symbol." ),
				};
				self.generate_value_track( s, &span )?
			},
			ast::ValueTrack::Sequence( ref ss ) => {
				let mut irs = Vec::new();
				let mut t = span.t0;
				for s in ss.iter() {
					let span = Span{
						t0: t,
						t1: t + (span.t1 - span.t0),
						.. *span
					};
					let (ir, t1) = self.generate_value_track( s, &span )?;
					irs.push( (ir, t) );
					t = t1;
				}
				(Ir::Sequence( irs ), t)
			},
			ast::ValueTrack::Stretch( ref s, r ) => {
				let span = Span{
					t1: span.t0 + r * (span.t1 - span.t0),
					.. *span
				};
				self.generate_value_track( s, &span )?
			},
			ast::ValueTrack::BinaryOp( ref lhs, ref rhs, op ) => {
				let (ir_lhs, t_lhs) = self.generate_value_track( lhs, &span )?;
				let (ir_rhs, t_rhs) = self.generate_value_track( rhs, &span )?;
				let ir = Ir::BinaryOp( Box::new( ir_lhs ), Box::new( ir_rhs ), op );
				let t = cmp::max( t_lhs, t_rhs );
				(ir, t)
			},
			ast::ValueTrack::Gaussian => {
				(Ir::Gaussian, span.t0)
			},
		};
		Ok( dst )
	}

	fn generate_value( &self, value: &ast::Ast<ast::Value>, span: &Span, state: &mut State, dst: &mut Vec<(Ir, ratio::Ratio)> ) -> Result<(), misc::Error> {
		match value.ast {
			ast::Value::Value( v0, v1 ) => {
				dst.push( (Ir::Value( span.t0, span.t1, v0, v1 ), span.t0) );
			},
			ast::Value::Group( ref vs ) => {
				let tot: i32 = vs.iter().map( |&(_, i)| i ).sum();
				let mut acc = 0;
				for &(ref v, i) in vs.iter() {
					let span = Span{
						t0: span.t0 + (span.t1 - span.t0) * ratio::Ratio::new( (acc    ) as i64, tot as i64 ),
						t1: span.t0 + (span.t1 - span.t0) * ratio::Ratio::new( (acc + i) as i64, tot as i64 ),
					};
					self.generate_value( v, &span, state, dst )?;
					acc += i;
				}
			},
		};
		Ok( () )
	}
}
