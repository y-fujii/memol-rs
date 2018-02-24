// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use random;
use ratio;
use ast;
use super::*;


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

	pub fn eval( &mut self, ir: &ValueIr, t: ratio::Ratio ) -> f64 {
		match *ir {
			ValueIr::Value( t0, t1, v0, v1 ) => {
				let t = cmp::min( cmp::max( t, t0 ), t1 );
				let v = v0 + (v1 - v0) * (t - t0) / (t1 - t0);
				v.to_float()
			},
			ValueIr::Symbol( ref sym ) => {
				let f = self.syms.get_mut( sym ).unwrap();
				f( t )
			},
			ValueIr::Sequence( ref irs ) => {
				let i = misc::bsearch_boundary( &irs, |&(_, t0)| t0 <= t );
				self.eval( &irs[i - 1].0, t )
			},
			ValueIr::BinaryOp( ref ir_lhs, ref ir_rhs, op ) => {
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
					ast::BinaryOp::Or => lhs + rhs - lhs * rhs,
				}
			},
			ValueIr::Branch( ref ir_cond, ref ir_then, ref ir_else ) => {
				let cond = self.eval( ir_cond, t );
				let then = self.eval( ir_then, t );
				let elze = self.eval( ir_else, t );
				cond * then + (1.0 - cond) * elze
			},
		}
	}
}

