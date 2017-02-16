// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use ratio;
use ast;


#[derive(Debug)]
pub struct FlatNote {
	pub bgn: ratio::Ratio,
	pub end: ratio::Ratio,
	pub nnum: Option<i32>,
}

#[derive(Debug)]
pub struct Ir {
	pub notes: Vec<FlatNote>,
	pub marks: Vec<ratio::Ratio>,
}

#[derive(Debug)]
struct Span<'a> {
	bgn: ratio::Ratio,
	end: ratio::Ratio,
	tied: bool,
	syms: &'a collections::HashMap<char, &'a [FlatNote]>,
}

#[derive(Debug)]
struct NoteState<'a> {
	nnum: i32,
	note: Option<&'a ast::Ast<ast::Note>>,
	ties: collections::HashMap<i32, ratio::Ratio>,
}

#[derive(Debug)]
pub struct Generator<'a> {
	defs: &'a ast::Definition,
	syms: Vec<(char, Vec<FlatNote>)>,
}

impl<'a> Generator<'a> {
	pub fn new( defs: &ast::Definition ) -> Generator {
		let syms = vec![ ('_', vec![
			FlatNote{ bgn: -ratio::Ratio::inf(), end: ratio::Ratio::inf(), nnum: Some( 69 ) },
			FlatNote{ bgn: -ratio::Ratio::inf(), end: ratio::Ratio::inf(), nnum: Some( 71 ) },
			FlatNote{ bgn: -ratio::Ratio::inf(), end: ratio::Ratio::inf(), nnum: Some( 60 ) },
			FlatNote{ bgn: -ratio::Ratio::inf(), end: ratio::Ratio::inf(), nnum: Some( 62 ) },
			FlatNote{ bgn: -ratio::Ratio::inf(), end: ratio::Ratio::inf(), nnum: Some( 64 ) },
			FlatNote{ bgn: -ratio::Ratio::inf(), end: ratio::Ratio::inf(), nnum: Some( 65 ) },
			FlatNote{ bgn: -ratio::Ratio::inf(), end: ratio::Ratio::inf(), nnum: Some( 67 ) },
		] ) ];

		Generator{ defs: defs, syms: syms }
	}

	pub fn generate( &self, key: &str ) -> Result<Option<Ir>, misc::Error> {
		let syms = self.syms.iter().map( |&(s, ref ns)| (s, &ns[..]) ).collect();
		let span = Span{
			bgn: ratio::Ratio::zero(),
			end: ratio::Ratio::zero(),
			tied: false,
			syms: &syms,
		};
		let s = match self.defs.scores.get( key ) {
			Some( v ) => v,
			None      => return Ok( None ),
		};
		let mut dst = Ir{
			notes: Vec::new(),
			marks: Vec::new(),
		};
		self.generate_score( s, &span, &mut dst )?;
		Ok( Some( dst ) )
	}

	fn generate_score( &self, score: &ast::Ast<ast::Score>, span: &Span, dst: &mut Ir ) -> Result<ratio::Ratio, misc::Error> {
		let end = match score.ast {
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
					return misc::error( score.end, "unpaired tie." );
				}
				span.bgn + ns.len() as i64
			}
			ast::Score::Symbol( ref key ) => {
				let s = match self.defs.scores.get( key ) {
					Some( v ) => v,
					None      => return misc::error( score.bgn, "undefined symbol." ),
				};
				self.generate_score( s, &span, dst )?
			},
			ast::Score::With( ref lhs, ref key, ref rhs ) => {
				let mut dst_rhs = Ir{
					notes: Vec::new(),
					marks: Vec::new(),
				};
				self.generate_score( rhs, &span, &mut dst_rhs )?;
				let mut syms = span.syms.clone();
				syms.insert( *key, &dst_rhs.notes[..] );
				dst.marks.extend( dst_rhs.marks );
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

	fn generate_note<'b>( &self, note: &'b ast::Ast<ast::Note>, span: &Span, state: &mut NoteState<'b>, dst: &mut Ir ) -> Result<(), misc::Error> {
		match note.ast {
			ast::Note::Note( ref dir, sym, ord, sig ) => {
				let fs = match span.syms.get( &sym ) {
					Some( v ) => v,
					None      => return misc::error( note.bgn, "note does not exist." ),
				};
				let f = match fs.iter().filter( |n| n.bgn <= span.bgn && span.bgn < n.end ).nth( ord as usize ) {
					Some( v ) => v,
					None      => return misc::error( note.bgn, "note does not exist." ),
				};
				let nnum = match f.nnum {
					Some( v ) => v,
					None => {
						dst.notes.push( FlatNote{
							bgn: span.bgn,
							end: span.end,
							nnum: None,
						} );
						return Ok( () );
					},
				};
				let nnum = match *dir {
					ast::Dir::Absolute( n ) => nnum + n * 12 + sig,
					ast::Dir::Lower => {
						let nnum = misc::idiv( state.nnum, 12 ) * 12 + misc::imod( nnum + sig, 12 );
						nnum - if nnum <= state.nnum { 0 } else { 12 }
					},
					ast::Dir::Upper => {
						let nnum = misc::idiv( state.nnum, 12 ) * 12 + misc::imod( nnum + sig, 12 );
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
					dst.notes.push( FlatNote{
						bgn: bgn,
						end: span.end,
						nnum: Some( nnum ),
					} );
				}
				state.nnum = nnum;
				state.note = Some( note );
			},
			ast::Note::Rest => {
				dst.notes.push( FlatNote{
					bgn: span.bgn,
					end: span.end,
					nnum: None,
				} );
			},
			ast::Note::Repeat => {
				match state.note {
					Some( n ) => self.generate_note( n, span, state, dst )?,
					None      => return misc::error( note.bgn, "previous note does not exist." ),
				}
			},
			ast::Note::Mark => {
				dst.marks.push( span.bgn );
			},
			ast::Note::Octave( oct ) => {
				state.nnum += oct * 12;
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
					if let None = state.ties.remove( k ) {
						return misc::error( note.bgn, "unpaired tie." );
					}
				}
				for &(k, v) in new_ties.iter() {
					if let Some( _ ) = state.ties.insert( k, v ) {
						return misc::error( note.end, "unpaired tie." );
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
						tied: acc + i == tot && span.tied, // only apply to the last note.
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
