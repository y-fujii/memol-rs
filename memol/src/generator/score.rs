// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use ratio;
use ast;
use super::*;


#[derive(Debug)]
pub struct ScoreIr {
	pub notes: Vec<FlatNote>,
}

struct State<'a> {
	nnum: i32,
	note: Option<&'a ast::Ast<ast::Note<'a>>>,
	ties: collections::HashMap<i32, ratio::Ratio>,
}

impl<'a> Generator<'a> {
	pub fn generate_score( &self, key: &str ) -> Result<Option<ScoreIr>, misc::Error> {
		let syms = self.syms.iter().map( |&(s, ref ns)| (s, &ns[..]) ).collect();
		let &(ref path, ref s) = match self.defs.scores.get( key ) {
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
		let mut dst = ScoreIr{
			notes: Vec::new(),
		};
		self.generate_score_inner( s, &span, &mut dst )?;
		Ok( Some( dst ) )
	}

	fn generate_score_inner( &self, score: &'a ast::Ast<ast::Score<'a>>, span: &Span, dst: &mut ScoreIr ) -> Result<ratio::Ratio, misc::Error> {
		let end = match score.ast {
			ast::Score::Score( ref ns ) => {
				let mut state = State{
					nnum: 60,
					note: None,
					ties: collections::HashMap::new(),
				};
				for (i, n) in ns.iter().enumerate() {
					let span = Span{
						t0: span.t0 + (span.t1 - span.t0) * i as i64,
						t1: span.t1 + (span.t1 - span.t0) * i as i64,
						.. *span
					};
					self.generate_score_note( n, &span, &mut state, dst )?;
				}
				if !state.ties.is_empty() {
					return misc::error( &span.path, score.end, "unpaired tie." );
				}
				span.t0 + (span.t1 - span.t0) * ns.len() as i64
			},
			ast::Score::Symbol( ref key ) => {
				let &(ref path, ref s) = match self.defs.scores.get( key ) {
					Some( v ) => v,
					None      => return misc::error( &span.path, score.bgn, "undefined symbol." ),
				};
				let span = Span{
					path: path,
					.. *span
				};
				self.generate_score_inner( s, &span, dst )?
			},
			ast::Score::With( ref lhs, ref key, ref rhs ) => {
				let mut dst_rhs = ScoreIr{
					notes: Vec::new(),
				};
				self.generate_score_inner( rhs, &span, &mut dst_rhs )?;
				let mut syms = span.syms.clone();
				syms.insert( *key, &dst_rhs.notes[..] );
				let span = Span{
					syms: &syms,
					.. *span
				};
				self.generate_score_inner( lhs, &span, dst )?
			},
			ast::Score::Parallel( ref ss ) => {
				let mut t = span.t0;
				for s in ss.iter() {
					t = cmp::max( t, self.generate_score_inner( s, &span, dst )? );
				}
				t
			},
			ast::Score::Sequence( ref ss ) => {
				let mut t = span.t0;
				for s in ss.iter() {
					let span = Span{
						t0: t,
						t1: t + (span.t1 - span.t0),
						.. *span
					};
					t = self.generate_score_inner( s, &span, dst )?;
				}
				t
			},
			ast::Score::Repeat( ref s, n ) => {
				let mut t = span.t0;
				for _ in 0 .. n {
					let span = Span{
						t0: t,
						t1: t + (span.t1 - span.t0),
						.. *span
					};
					t = self.generate_score_inner( s, &span, dst )?;
				}
				t
			},
			ast::Score::Stretch( ref s, r ) => {
				let span = Span{
					t1: span.t0 + r * (span.t1 - span.t0),
					.. *span
				};
				self.generate_score_inner( s, &span, dst )?
			},
			_ => {
				return misc::error( &span.path, score.bgn, "syntax error." );
			}
		};
		Ok( end )
	}

	fn generate_score_note( &self, note: &'a ast::Ast<ast::Note<'a>>, span: &Span, state: &mut State<'a>, dst: &mut ScoreIr ) -> Result<(), misc::Error> {
		match note.ast {
			ast::Note::Note( dir, sym, ord, sig ) => {
				let nnum = match self.get_nnum( note, span, sym, ord )? {
					Some( v ) => v,
					None => {
						dst.notes.push( FlatNote{
							t0: span.t0,
							t1: span.t1,
							nnum: None,
						} );
						return Ok( () );
					},
				};
				let nnum = misc::idiv( state.nnum, 12 ) * 12 + misc::imod( nnum + sig, 12 );
				let nnum = nnum + match dir {
					ast::Dir::Lower => if nnum <= state.nnum { 0 } else { -12 },
					ast::Dir::Upper => if nnum >= state.nnum { 0 } else {  12 },
				};
				let t0 = match state.ties.remove( &nnum ) {
					Some( v ) => v,
					None      => span.t0,
				};
				if span.tied {
					state.ties.insert( nnum, t0 );
				}
				else {
					dst.notes.push( FlatNote{
						t0: t0,
						t1: span.t1,
						nnum: Some( nnum ),
					} );
				}
				state.nnum = nnum;
				state.note = Some( note );
			},
			ast::Note::Rest => {
				dst.notes.push( FlatNote{
					t0: span.t0,
					t1: span.t1,
					nnum: None,
				} );
			},
			ast::Note::Repeat( ref cn ) => {
				let rn = match cn.get() {
					Some( n ) => n,
					None => match state.note {
						Some( n ) => n,
						None      => return misc::error( &span.path, note.bgn, "previous note does not exist." ),
					}
				};
				cn.set( Some( rn ) );
				self.generate_score_note( rn, span, state, dst )?
			},
			ast::Note::Octave( oct ) => {
				state.nnum += oct * 12;
			},
			ast::Note::OctaveByNote( sym, ord, sig ) => {
				if let Some( v ) = self.get_nnum( note, span, sym, ord )? {
					state.nnum = v + sig;
				}
			},
			ast::Note::Chord( ref ns ) => {
				let mut del_ties = Vec::new();
				let mut new_ties = Vec::new();
				let mut s = State{
					ties: collections::HashMap::new(),
					.. *state
				};
				for (i, n) in ns.iter().enumerate() {
					s.ties = state.ties.clone();
					self.generate_score_note( n, span, &mut s, dst )?;
					for k in state.ties.keys() {
						match s.ties.get( k ) {
							Some( v ) if *v < span.t0 => (),
							_ => del_ties.push( *k ),
						}
					}
					for (k, v) in s.ties.iter() {
						if *v >= span.t0 {
							new_ties.push( (*k, *v) );
						}
					}
					if i == 0 {
						state.nnum = s.nnum;
					}
				}
				state.note = Some( note );
				for k in del_ties.iter() {
					if let None = state.ties.remove( k ) {
						return misc::error( &span.path, note.bgn, "unpaired tie." );
					}
				}
				for &(k, v) in new_ties.iter() {
					if let Some( _ ) = state.ties.insert( k, v ) {
						return misc::error( &span.path, note.end, "unpaired tie." );
					}
				}
			},
			ast::Note::Group( ref ns ) => {
				let tot = ns.iter().map( |&(_, i)| i ).sum();
				if tot == 0 {
					return misc::error( &span.path, note.end, "zero length group." );
				}
				let mut acc = 0;
				for &(ref n, i) in ns.iter() {
					let span = Span{
						t0: span.t0 + (span.t1 - span.t0) * ratio::Ratio::new( (acc    ) as i64, tot as i64 ),
						t1: span.t0 + (span.t1 - span.t0) * ratio::Ratio::new( (acc + i) as i64, tot as i64 ),
						tied: acc + i == tot && span.tied, // only apply to the last note.
						.. *span
					};
					self.generate_score_note( n, &span, state, dst )?;
					acc += i;
				}
			},
			ast::Note::Tie( ref n ) => {
				let span = Span{
					tied: true,
					.. *span
				};
				self.generate_score_note( n, &span, state, dst )?
			},
			_ => {
				return misc::error( &span.path, note.bgn, "syntax error." );
			},
		};
		Ok( () )
	}

	fn get_nnum( &self, note: &'a ast::Ast<ast::Note<'a>>, span: &Span, sym: char, ord: i32 ) -> Result<Option<i32>, misc::Error> {
		let fs = match span.syms.get( &sym ) {
			Some( v ) => v,
			None      => return misc::error( &span.path, note.bgn, "note does not exist." ),
		};
		// XXX: O(N^2).
		let f = match fs.iter().filter( |n| n.t0 <= span.t0 && span.t0 < n.t1 ).nth( ord as usize ) {
			Some( v ) => v,
			None      => return misc::error( &span.path, note.bgn, "note does not exist." ),
		};
		Ok( f.nnum )
	}
}
