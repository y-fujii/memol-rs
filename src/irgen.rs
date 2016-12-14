// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD License.
use std::*;
use misc;
use ratio;
use ast;


#[derive(Copy, Clone, Debug)]
pub struct FlatNote {
	pub bgn: ratio::Ratio,
	pub end: ratio::Ratio,
	pub nnum: i32,
}

#[derive(Debug)]
struct Span<'a> {
	bgn: ratio::Ratio,
	end: ratio::Ratio,
	tied: bool,
	syms: &'a collections::HashMap<char, Vec<FlatNote>>,
}

#[derive(Debug)]
struct NoteState<'a> {
	nnum: i32,
	note: Option<&'a ast::Note>,
	ties: collections::HashMap<i32, ratio::Ratio>,
}

#[derive(Debug)]
pub struct Generator<'a> {
	defs: &'a ast::Definition,
	syms: collections::HashMap<char, Vec<FlatNote>>,
}

impl<'a> Generator<'a> {
	pub fn new( defs: &ast::Definition ) -> Generator {
		let ninf = ratio::Ratio::new( -1, 0 );
		let pinf = ratio::Ratio::new(  1, 0 );
		let mut syms = collections::HashMap::new();
		syms.insert( '_', vec![
			FlatNote{ bgn: ninf, end: pinf, nnum:  9 },
			FlatNote{ bgn: ninf, end: pinf, nnum: 11 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  0 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  2 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  4 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  5 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  7 },
		] );

		Generator{ defs: defs, syms: syms }
	}

	pub fn generate( &self, key: &str ) -> Result<Vec<FlatNote>, misc::Error> {
		let span = Span{
			bgn: ratio::Ratio::new( 0, 1 ),
			end: ratio::Ratio::new( 0, 1 ),
			tied: false,
			syms: &self.syms,
		};
		let s = match self.defs.scores.iter().find( |&&(ref k, _)| k == key ) {
			Some( &(_, ref v) ) => v,
			None                => return misc::Error::new( "" ),
		};
		let mut dst = Vec::new();
		self.generate_score( s, &span, &mut dst )?;
		Ok( dst )
	}

	fn generate_score( &self, score: &ast::Score, span: &Span, dst: &mut Vec<FlatNote> ) -> Result<ratio::Ratio, misc::Error> {
		let end = match *score {
			ast::Score::Score( ref ns ) => {
				let mut state = NoteState{
					nnum: 60,
					note: None,
					ties: collections::HashMap::new(),
				};
				for (i, n) in ns.iter().enumerate() {
					let span = Span{
						bgn: span.bgn + i as i64,
						end: span.bgn + i as i64 + 1,
						tied: false,
						syms: span.syms,
					};
					self.generate_note( n, &span, &mut state, dst )?;
				}
				if !state.ties.is_empty() {
					return misc::Error::new( "" );
				}
				span.bgn + ns.len() as i64
			}
			ast::Score::Variable( ref key ) => {
				let s = match self.defs.scores.iter().find( |&&(ref k, _)| k == key ) {
					Some( &(_, ref v) ) => v,
					None                => return misc::Error::new( "" ),
				};
				self.generate_score( s, &span, dst )?
			},
			ast::Score::With( ref lhs, ref key, ref rhs ) => {
				let mut dst_rhs = Vec::new();
				self.generate_score( rhs, &span, &mut dst_rhs )?;
				let mut syms = span.syms.clone(); // XXX
				syms.insert( *key, dst_rhs );
				let span = Span{
					bgn: span.bgn,
					end: span.end,
					tied: false,
					syms: &syms,
				};
				self.generate_score( lhs, &span, dst )?
			},
			ast::Score::Parallel( ref ss ) => {
				let mut t = span.bgn;
				for s in ss.iter() {
					let span = Span{
						bgn: span.bgn,
						end: span.end,
						tied: false,
						syms: span.syms,
					};
					t = t.max( self.generate_score( s, &span, dst )? );
				}
				t
			},
			ast::Score::Sequence( ref ss ) => {
				let mut t = span.bgn;
				for s in ss.iter() {
					let span = Span{
						bgn: t,
						end: t,
						tied: false,
						syms: span.syms,
					};
					t = self.generate_score( s, &span, dst )?;
				}
				t
			},
		};
		Ok( end )
	}

	fn generate_note<'b>( &self, note: &'b ast::Note, span: &Span, state: &mut NoteState<'b>, dst: &mut Vec<FlatNote> ) -> Result<(), misc::Error> {
		match *note {
			ast::Note::Note( ref dir, sym, ord, sig ) => {
				let fs = match span.syms.get( &sym ) {
					Some( v ) => v,
					None      => return misc::Error::new( "" ),
				};
				let f = match fs.iter().filter( |n| n.bgn <= span.bgn && span.bgn < n.end ).nth( ord as usize ) {
					Some( v ) => v,
					None      => return misc::Error::new( "" ),
				};
				let nnum = match *dir {
					ast::Dir::Absolute( n ) => f.nnum + n * 12 + sig,
					ast::Dir::Lower => {
						let nnum = state.nnum / 12 * 12 + (f.nnum + sig) % 12;
						nnum - if nnum <= state.nnum { 0 } else { 12 }
					},
					ast::Dir::Upper => {
						let nnum = state.nnum / 12 * 12 + (f.nnum + sig) % 12;
						nnum + if nnum >= state.nnum { 0 } else { 12 }
					},
				};
				let bgn = match state.ties.remove( &nnum ) {
					Some( v ) => v,
					None      => span.bgn,
				};
				if span.tied {
					state.ties.insert( nnum, bgn );
				}
				else {
					dst.push( FlatNote{
						bgn: bgn,
						end: span.end,
						nnum: nnum,
					} );
				}
				state.nnum = nnum;
				state.note = Some( note );
			},
			ast::Note::Rest => {
			},
			ast::Note::Repeat => {
				match state.note {
					Some( n ) => self.generate_note( n, span, state, dst )?,
					None      => return misc::Error::new( "" ),
				}
			},
			ast::Note::Octave( oct ) => {
				state.nnum += oct * 12
			},
			ast::Note::Chord( ref ns ) => {
				let mut del_ties = Vec::new();
				let mut new_ties = Vec::new();
				let mut s = NoteState{
					nnum: state.nnum,
					note: state.note,
					ties: collections::HashMap::new(),
				};
				for (i, n) in ns.iter().enumerate() {
					s.ties = state.ties.clone();
					self.generate_note( n, span, &mut s, dst )?;
					for k in state.ties.keys() {
						match s.ties.get( k ) {
							Some( v ) if *v < span.bgn => (),
							_ => del_ties.push( *k ),
						}
					}
					for (k, v) in s.ties.iter() {
						match state.ties.get( k ) {
							Some( v ) if *v < span.bgn => (),
							_ => new_ties.push( (*k, *v) ),
						}
					}
					if i == 0 {
						state.nnum = s.nnum;
					}
				}
				state.note = Some( note );
				for k in del_ties.iter() {
					if state.ties.remove( k ) == None {
						return misc::Error::new( "" );
					}
				}
				for &(k, v) in new_ties.iter() {
					if state.ties.insert( k, v ) != None {
						return misc::Error::new( "" );
					}
				}
			},
			ast::Note::Group( ref ns ) => {
				let tot: i32 = ns.iter().map( |&(_, i)| i ).sum();
				let mut acc = 0;
				for &(ref n, i) in ns.iter() {
					let span = Span{
						bgn: span.bgn + (span.end - span.bgn) * ratio::Ratio::new( (acc    ) as i64, tot as i64 ),
						end: span.bgn + (span.end - span.bgn) * ratio::Ratio::new( (acc + i) as i64, tot as i64 ),
						tied: acc == 0 && span.tied,
						syms: span.syms,
					};
					self.generate_note( n, &span, state, dst )?;
					acc += i;
				}
			},
			ast::Note::Tie( ref n ) => {
				let span = Span{
					bgn: span.bgn,
					end: span.end,
					tied: true,
					syms: span.syms,
				};
				self.generate_note( n, &span, state, dst )?
			},
		};
		Ok( () )
	}
}
