// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use crate::misc;
use crate::random;
use crate::ratio;
use crate::ast;
use super::*;


pub struct Evaluator<'a> {
	rng: &'a random::Generator,
	pub note_len: f64,
	pub note_cnt: f64,
	pub note_nth: f64,
}

impl<'a> Evaluator<'a> {
	pub fn new( rng: &'a random::Generator ) -> Self {
		Evaluator{
			rng: rng,
			note_len: 0.0,
			note_cnt: 0.0,
			note_nth: 0.0,
		}
	}

	pub fn set_note( &mut self, ir: &ScoreIr, f: &FlatNote ) {
		// XXX: O(N^2).
		let mut cnt = 0;
		for g in ir.iter().filter( |g| g.t0 <= f.t0 && f.t0 < g.t1 ) {
			if g as *const _ == f as *const _ {
				self.note_nth = cnt as f64;
			}
			cnt += 1;
		}
		self.note_cnt = cnt as f64;
		self.note_len = (f.t1 - f.t0).to_float();
	}

	pub fn eval( &self, ir: &ValueIr, t: ratio::Ratio ) -> f64 {
		match *ir {
			ValueIr::Value( t0, t1, v0, v1 ) => {
				let t = cmp::min( cmp::max( t, t0 ), t1 );
				let v = v0 + (v1 - v0) * (t - t0) / (t1 - t0);
				v.to_float()
			},
			ValueIr::Sequence( t0, ref irs ) => {
				let t = cmp::min( cmp::max( t, t0 ), irs.last().unwrap().1 );
				let i = misc::bsearch_boundary( &irs, |&(_, t1)| t1 <= t );
				let i = cmp::min( i, irs.len() - 1 );
				self.eval( &irs[i].0, t )
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
			ValueIr::Time    => t.to_float(),
			ValueIr::Gauss   => self.rng.next_gauss(),
			ValueIr::NoteLen => self.note_len,
			ValueIr::NoteCnt => self.note_cnt,
			ValueIr::NoteNth => self.note_nth,
		}
	}
}
