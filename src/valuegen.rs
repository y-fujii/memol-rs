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
	Symbol( String ),
}

#[derive(Debug)]
struct Span {
	t0: ratio::Ratio,
	t1: ratio::Ratio,
}

#[derive(Debug)]
struct State {
}

pub struct Generator<'a> {
	defs: &'a ast::Definition<'a>,
	syms: collections::HashSet<String>,
}

impl<'a> Generator<'a> {
	pub fn new( defs: &'a ast::Definition<'a> ) -> Generator<'a> {
		let mut syms = collections::HashSet::new();
		syms.insert( "gaussian".into() );
		syms.insert( "note_len".into() );
		Generator{
			defs: defs,
			syms: syms,
		}
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
				if let Some( s ) = self.defs.values.get( key ) {
					self.generate_value_track( s, &span )?
				}
				else if self.syms.contains( key ) {
					(Ir::Symbol( key.clone() ), span.t0)
				}
				else {
					return misc::error( track.bgn, "undefined symbol." );
				}
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

pub struct Evaluator<'a> {
	syms: collections::HashMap<String, Box<Fn( ratio::Ratio ) -> f64 + 'a>>,
}

impl<'a> Evaluator<'a> {
	pub fn new() -> Self {
		let mut this = Evaluator{ syms: collections::HashMap::new() };
		this.add_symbol( "gaussian".into(), |_| rand::random::<rand::distributions::normal::StandardNormal>().0 );
		this.add_symbol( "note_len".into(), |_| 0.0 );
		this
	}

	pub fn add_symbol<F: Fn( ratio::Ratio ) -> f64 + 'a>( &mut self, key: String, f: F ) {
		self.syms.insert( key, Box::new( f ) );
	}

	pub fn eval( &self, ir: &Ir, t: ratio::Ratio ) -> f64 {
		match *ir {
			Ir::Value( t0, t1, v0, v1 ) => {
				let t = cmp::min( cmp::max( t, t0 ), t1 );
				let v = v0 + (v1 - v0) * (t - t0) / (t1 - t0);
				v.to_float()
			},
			Ir::Sequence( ref irs ) => {
				let i = misc::bsearch_boundary( &irs, |&(_, t0)| t0 <= t );
				self.eval( &irs[i - 1].0, t )
			},
			Ir::BinaryOp( ref ir_lhs, ref ir_rhs, op ) => {
				let lhs = self.eval( ir_lhs, t );
				let rhs = self.eval( ir_rhs, t );
				match op {
					ast::BinaryOp::Add => lhs + rhs,
					ast::BinaryOp::Sub => lhs - rhs,
					ast::BinaryOp::Mul => lhs * rhs,
					ast::BinaryOp::Div => lhs / rhs,
				}
			},
			Ir::Symbol( ref sym ) => {
				let f = self.syms.get( sym ).unwrap();
				f( t )
			},
		}
	}
}

