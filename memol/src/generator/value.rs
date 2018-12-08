// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use crate::misc;
use crate::ratio::Ratio;
use crate::ast;
use super::*;


#[derive( Debug )]
pub enum ValueIr {
	Value( Ratio, Ratio, Ratio, Ratio ),
	Sequence( Ratio, Vec<(ValueIr, Ratio)> ),
	BinaryOp( Box<ValueIr>, Box<ValueIr>, ast::BinaryOp ),
	Branch( Box<ValueIr>, Box<ValueIr>, Box<ValueIr> ),
	Time,
	Gauss,
	NoteLen,
	NoteCnt,
	NoteNth,
}

pub struct ValueState<'a> {
	t: Ratio,
	v: Ratio,
	note: Option<&'a ast::Ast<ast::Note<'a>>>,
}

impl<'a> Generator<'a> {
	pub fn generate_value( &self, key: &str ) -> Result<Option<ValueIr>, misc::Error> {
		let syms = self.syms.iter().map( |&(s, ref ns)| (s, &ns[..]) ).collect();
		let &(ref path, ref s) = match self.defs.values.get( key ) {
			Some( v ) => v,
			None      => return Ok( None ),
		};
		let span = Span{
			t0: Ratio::zero(),
			dt: Ratio::one(),
			tied: false,
			syms: &syms,
			path: path,
		};
		let (ir, _) = self.generate_value_inner( s, &span )?;
		Ok( Some( ir ) )
	}

	pub fn generate_value_inner( &self, track: &'a ast::Ast<ast::Score<'a>>, span: &Span ) -> Result<(ValueIr, Ratio), misc::Error> {
		let dst = match track.ast {
			ast::Score::Score( ref ns ) => {
				let mut irs = Vec::new();
				let mut state = ValueState{
					t: span.t0,
					v: Ratio::zero(),
					note: None,
				};
				for (i, v) in ns.iter().enumerate() {
					let span = Span{ t0: span.t0 + span.dt * i as i64, .. *span };
					self.generate_value_note( v, &span, &mut state, &mut irs )?;
				}
				let t1 = span.t0 + span.dt * ns.len() as i64;
				if state.t != t1 {
					return misc::error( &span.path, track.end, "the last value must be specified." );
				}
				(ValueIr::Sequence( span.t0, irs ), t1)
			},
			ast::Score::Symbol( ref key ) => {
				match key.as_str() {
					"time"     => (ValueIr::Time,    span.t0),
					"gauss"    => (ValueIr::Gauss,   span.t0),
					"note.len" => (ValueIr::NoteLen, span.t0),
					"note.cnt" => (ValueIr::NoteCnt, span.t0),
					"note.nth" => (ValueIr::NoteNth, span.t0),
					_ => {
						let &(ref path, ref s) = match self.defs.values.get( key ) {
							Some( v ) => v,
							None      => return misc::error( &span.path, track.bgn, "undefined symbol." ),
						};
						let span = Span{ path: path, .. *span };
						self.generate_value_inner( s, &span )?
					}
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
					let span = Span{ t0: t, .. *span };
					let (ir, t1) = self.generate_value_inner( s, &span )?;
					irs.push( (ir, t1) );
					t = t1;
				}
				(ValueIr::Sequence( span.t0, irs ), t)
			},
			ast::Score::Repeat( ref s, n ) => {
				let mut irs = Vec::new();
				let mut t = span.t0;
				for _ in 0 .. n {
					let span = Span{ t0: t, .. *span };
					let (ir, t1) = self.generate_value_inner( s, &span )?;
					irs.push( (ir, t1) );
					t = t1;
				}
				(ValueIr::Sequence( span.t0, irs ), t)
			},
			ast::Score::Stretch( ref s, r ) => {
				let span = Span{ dt: r * span.dt, .. *span };
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
			ast::Score::Slice( ref s, t0, t1 ) => {
				let span1 = Span{ t0: span.t0 - t0, .. *span };
				let (ir, _) = self.generate_value_inner( s, &span1 )?;
				let t = span.t0 + (t1 - t0);
				(ValueIr::Sequence( span.t0, vec![ (ir, t) ] ), t)
			},
			_ => {
				return misc::error( &span.path, track.bgn, "syntax error." );
			},
		};
		Ok( dst )
	}

	pub fn generate_value_note( &self, note: &'a ast::Ast<ast::Note<'a>>, span: &Span, state: &mut ValueState<'a>, dst: &mut Vec<(ValueIr, Ratio)> ) -> Result<(), misc::Error> {
		match note.ast {
			ast::Note::Value( v0, v1 ) => {
				if let Some( v0 ) = v0 {
					if state.t != span.t0 {
						dst.push( (ValueIr::Value( state.t, span.t0, state.v, v0 ), span.t0) );
					}
					state.t = span.t0;
					state.v = v0;
				}
				if let Some( v1 ) = v1 {
					let t1 = span.t0 + span.dt;
					dst.push( (ValueIr::Value( state.t, t1, state.v, v1 ), t1) );
					state.t = t1;
					state.v = v1;
				}
				state.note = Some( note );
			},
			ast::Note::Repeat( ref cn ) => {
				let rn = match cn.get() {
					Some( n ) => n,
					None => match state.note {
						Some( n ) => n,
						None      => return misc::error( &span.path, note.bgn, "previous note does not exist." ),
					},
				};
				cn.set( Some( rn ) );
				self.generate_value_note( rn, span, state, dst )?
			},
			ast::Note::Chord( ref ns ) => {
				let mut acc = 0;
				for &(ref n, i) in ns.iter() {
					if acc > 0 {
						return misc::error( &span.path, note.bgn, "syntax error." );
					}
					self.generate_value_note( n, span, state, dst )?;
					acc += i;
				}
				state.note = Some( note );
			},
			ast::Note::Group( ref ns ) => {
				let tot = ns.iter().map( |&(_, i)| i ).sum();
				let mut acc = 0;
				for &(ref n, i) in ns.iter() {
					let span = Span{
						t0: span.t0 + span.dt * Ratio::new( acc, tot ),
						dt: span.dt * Ratio::new( i, tot ),
						.. *span
					};
					self.generate_value_note( n, &span, state, dst )?;
					acc += i;
				}
			},
			_ => {
				return misc::error( &span.path, note.bgn, "syntax error." );
			},
		}
		Ok( () )
	}
}
