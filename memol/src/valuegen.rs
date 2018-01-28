// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use random;
use ratio;
use ast;


#[derive(Debug)]
pub enum Ir {
	Value( ratio::Ratio, ratio::Ratio, ratio::Ratio, ratio::Ratio ),
	Symbol( String ),
	Sequence( Vec<(Ir, ratio::Ratio)> ),
	BinaryOp( Box<Ir>, Box<Ir>, ast::BinaryOp ),
	Branch( Box<Ir>, Box<Ir>, Box<Ir> ),
}

#[derive(Debug)]
struct Span<'a> {
	t0: ratio::Ratio,
	t1: ratio::Ratio,
	path: &'a path::Path,
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
		syms.insert( "note.len".into() );
		syms.insert( "note.cnt".into() );
		syms.insert( "note.nth".into() );
		Generator{
			defs: defs,
			syms: syms,
		}
	}

	pub fn generate( &self, key: &str ) -> Result<Option<Ir>, misc::Error> {
		let &(ref path, ref s) = match self.defs.values.get( key ) {
			Some( v ) => v,
			None      => return Ok( None ),
		};
		let span = Span{
			t0: ratio::Ratio::zero(),
			t1: ratio::Ratio::one(),
			path: path,
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
				if let Some( &(ref path, ref s) ) = self.defs.values.get( key ) {
					let span = Span{
						path: path,
						.. *span
					};
					self.generate_value_track( s, &span )?
				}
				else if self.syms.contains( key ) {
					(Ir::Symbol( key.clone() ), span.t0)
				}
				else {
					return misc::error( &span.path, track.bgn, "undefined symbol." );
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
			ast::ValueTrack::Repeat( ref s, n ) => {
				let mut irs = Vec::new();
				let mut t = span.t0;
				for _ in 0 .. n {
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
			ast::ValueTrack::Branch( ref cond, ref then, ref elze ) => {
				let (ir_cond, t_cond) = self.generate_value_track( cond, &span )?;
				let (ir_then, t_then) = self.generate_value_track( then, &span )?;
				let (ir_elze, t_elze) = self.generate_value_track( elze, &span )?;
				let ir = Ir::Branch( Box::new( ir_cond ), Box::new( ir_then ), Box::new( ir_elze ) );
				let t = cmp::max( t_cond, cmp::max( t_then, t_elze ) );
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
						.. *span
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
	syms: collections::HashMap<String, Box<'a + FnMut( ratio::Ratio ) -> f64>>,
}

impl<'a> Evaluator<'a> {
	pub fn new() -> Self {
		let mut this = Evaluator{
			syms: collections::HashMap::new(),
		};
		this.add_symbol( "gaussian".into(), move |_| 0.0 );
		this.add_symbol( "note.len".into(), move |_| 0.0 );
		this.add_symbol( "note.cnt".into(), move |_| 0.0 );
		this.add_symbol( "note.nth".into(), move |_| 0.0 );
		this
	}

	pub fn new_with_random( rng: &'a mut random::Generator ) -> Self {
		let mut this = Self::new();
		this.add_symbol( "gaussian".into(), move |_| rng.next_gauss() );
		this
	}

	pub fn add_symbol<F: 'a + FnMut( ratio::Ratio ) -> f64>( &mut self, key: String, f: F ) {
		self.syms.insert( key, Box::new( f ) );
	}

	pub fn eval( &mut self, ir: &Ir, t: ratio::Ratio ) -> f64 {
		match *ir {
			Ir::Value( t0, t1, v0, v1 ) => {
				let t = cmp::min( cmp::max( t, t0 ), t1 );
				let v = v0 + (v1 - v0) * (t - t0) / (t1 - t0);
				v.to_float()
			},
			Ir::Symbol( ref sym ) => {
				let f = self.syms.get_mut( sym ).unwrap();
				f( t )
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
					ast::BinaryOp::Eq => if lhs == rhs { 1.0 } else { 0.0 },
					ast::BinaryOp::Ne => if lhs != rhs { 1.0 } else { 0.0 },
					ast::BinaryOp::Le => if lhs <= rhs { 1.0 } else { 0.0 },
					ast::BinaryOp::Ge => if lhs >= rhs { 1.0 } else { 0.0 },
					ast::BinaryOp::Lt => if lhs <  rhs { 1.0 } else { 0.0 },
					ast::BinaryOp::Gt => if lhs >  rhs { 1.0 } else { 0.0 },
					ast::BinaryOp::Or => 1.0 - (1.0 - lhs) * (1.0 - rhs),
				}
			},
			Ir::Branch( ref ir_cond, ref ir_then, ref ir_else ) => {
				let cond = self.eval( ir_cond, t );
				let then = self.eval( ir_then, t );
				let elze = self.eval( ir_else, t );
				cond * then + (1.0 - cond) * elze
			},
		}
	}
}

