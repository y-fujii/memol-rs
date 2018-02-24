// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use ratio;
use ast;
use super::*;


#[derive( Debug )]
pub enum ValueIr {
	Value( ratio::Ratio, ratio::Ratio, ratio::Ratio, ratio::Ratio ),
	Symbol( String ),
	Sequence( Vec<(ValueIr, ratio::Ratio)> ),
	BinaryOp( Box<ValueIr>, Box<ValueIr>, ast::BinaryOp ),
	Branch( Box<ValueIr>, Box<ValueIr>, Box<ValueIr> ),
}

pub struct ValueState {
	t: ratio::Ratio,
	v: ratio::Ratio,
}

impl<'a> Generator<'a> {
	pub fn generate_value( &self, key: &str ) -> Result<Option<ValueIr>, misc::Error> {
		let syms = self.syms.iter().map( |&(s, ref ns)| (s, &ns[..]) ).collect();
		let &(ref path, ref s) = match self.defs.values.get( key ) {
			Some( v ) => v,
			None      => return Ok( None ),
		};
		let span = Span{
			t0: ratio::Ratio::zero(),
			t1: ratio::Ratio::one(),
			tied: false,
			syms: &syms,
			path: path,
		};
		let (ir, _) = self.generate_value_inner( s, &span )?;
		Ok( Some( ir ) )
	}

	pub fn generate_value_inner( &self, track: &'a ast::Ast<ast::Score<'a>>, span: &Span ) -> Result<(ValueIr, ratio::Ratio), misc::Error> {
		let dst = match track.ast {
			ast::Score::Score( ref vs ) => {
				let mut irs = Vec::new();
				let mut state = ValueState{
					t: span.t0,
					v: ratio::Ratio::zero(),
				};
				for (i, v) in vs.iter().enumerate() {
					let span = Span{
						t0: span.t0 + (span.t1 - span.t0) * i as i64,
						t1: span.t1 + (span.t1 - span.t0) * i as i64,
						.. *span
					};
					self.generate_value_note( v, &span, &mut state, &mut irs )?;
				}
				let t1 = span.t0 + (span.t1 - span.t0) * vs.len() as i64;
				if state.t != t1 {
					return misc::error( &span.path, track.end, "the last value must be specified." );
				}
				(ValueIr::Sequence( irs ), t1)
			},
			ast::Score::Symbol( ref key ) => {
				if let Some( &(ref path, ref s) ) = self.defs.values.get( key ) {
					let span = Span{
						path: path,
						.. *span
					};
					self.generate_value_inner( s, &span )?
				}
				else if self.values.contains( key ) {
					(ValueIr::Symbol( key.clone() ), span.t0)
				}
				else {
					return misc::error( &span.path, track.bgn, "undefined symbol." );
				}
			},
			ast::Score::Parallel( ref ss ) => {
				if ss.len() != 1 {
					return misc::error( &span.path, track.bgn, "syntax error." );
				}
				self.generate_value_inner( &ss[0], &span )?
			},
			ast::Score::Sequence( ref ss ) => {
				let mut irs = Vec::new();
				let mut t = span.t0;
				for s in ss.iter() {
					let span = Span{
						t0: t,
						t1: t + (span.t1 - span.t0),
						.. *span
					};
					let (ir, t1) = self.generate_value_inner( s, &span )?;
					irs.push( (ir, t) );
					t = t1;
				}
				(ValueIr::Sequence( irs ), t)
			},
			ast::Score::Repeat( ref s, n ) => {
				let mut irs = Vec::new();
				let mut t = span.t0;
				for _ in 0 .. n {
					let span = Span{
						t0: t,
						t1: t + (span.t1 - span.t0),
						.. *span
					};
					let (ir, t1) = self.generate_value_inner( s, &span )?;
					irs.push( (ir, t) );
					t = t1;
				}
				(ValueIr::Sequence( irs ), t)
			},
			ast::Score::Stretch( ref s, r ) => {
				let span = Span{
					t1: span.t0 + r * (span.t1 - span.t0),
					.. *span
				};
				self.generate_value_inner( s, &span )?
			},
			ast::Score::BinaryOp( ref lhs, ref rhs, op ) => {
				let (ir_lhs, t_lhs) = self.generate_value_inner( lhs, &span )?;
				let (ir_rhs, t_rhs) = self.generate_value_inner( rhs, &span )?;
				let ir = ValueIr::BinaryOp( Box::new( ir_lhs ), Box::new( ir_rhs ), op );
				let t = cmp::max( t_lhs, t_rhs );
				(ir, t)
			},
			ast::Score::Branch( ref cond, ref then, ref elze ) => {
				let (ir_cond, _     ) = self.generate_value_inner( cond, &span )?;
				let (ir_then, t_then) = self.generate_value_inner( then, &span )?;
				let (ir_elze, t_elze) = self.generate_value_inner( elze, &span )?;
				let ir = ValueIr::Branch( Box::new( ir_cond ), Box::new( ir_then ), Box::new( ir_elze ) );
				let t = cmp::max( t_then, t_elze );
				(ir, t)
			},
			_ => {
				return misc::error( &span.path, track.bgn, "syntax error." );
			},
		};
		Ok( dst )
	}

	pub fn generate_value_note( &self, value: &'a ast::Ast<ast::Note<'a>>, span: &Span, state: &mut ValueState, dst: &mut Vec<(ValueIr, ratio::Ratio)> ) -> Result<(), misc::Error> {
		match value.ast {
			ast::Note::Value( v0, v1 ) => {
				if let Some( v0 ) = v0 {
					if state.t != span.t0 {
						dst.push( (ValueIr::Value( state.t, span.t0, state.v, v0 ), state.t) );
					}
					state.t = span.t0;
					state.v = v0;
				}
				if let Some( v1 ) = v1 {
					dst.push( (ValueIr::Value( state.t, span.t1, state.v, v1 ), state.t) );
					state.t = span.t1;
					state.v = v1;
				}
			},
			ast::Note::Group( ref vs ) => {
				let tot: i32 = vs.iter().map( |&(_, i)| i ).sum();
				let mut acc = 0;
				for &(ref v, i) in vs.iter() {
					let span = Span{
						t0: span.t0 + (span.t1 - span.t0) * ratio::Ratio::new( (acc    ) as i64, tot as i64 ),
						t1: span.t0 + (span.t1 - span.t0) * ratio::Ratio::new( (acc + i) as i64, tot as i64 ),
						.. *span
					};
					self.generate_value_note( v, &span, state, dst )?;
					acc += i;
				}
			},
			_ => {
				return misc::error( &span.path, value.bgn, "syntax error." );
			},
		}
		Ok( () )
	}
}
