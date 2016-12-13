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
	pub tied: bool,
	/*
	tie_bgn: bool,
	tie_end: bool,
	*/
}

#[derive(Debug)]
struct Span<'a> {
	bgn: ratio::Ratio,
	end: ratio::Ratio,
	nnum: i32,
	tied: bool,
	syms: &'a collections::HashMap<char, Vec<FlatNote>>,
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
			FlatNote{ bgn: ninf, end: pinf, nnum:  9, tied: false },
			FlatNote{ bgn: ninf, end: pinf, nnum: 11, tied: false },
			FlatNote{ bgn: ninf, end: pinf, nnum:  0, tied: false },
			FlatNote{ bgn: ninf, end: pinf, nnum:  2, tied: false },
			FlatNote{ bgn: ninf, end: pinf, nnum:  4, tied: false },
			FlatNote{ bgn: ninf, end: pinf, nnum:  5, tied: false },
			FlatNote{ bgn: ninf, end: pinf, nnum:  7, tied: false },
		] );

		Generator{ defs: defs, syms: syms }
	}

	pub fn generate( &self, key: &str ) -> Result<Vec<FlatNote>, misc::Error> {
		let span = Span{
			bgn: ratio::Ratio::new( 0, 1 ),
			end: ratio::Ratio::new( 0, 1 ),
			nnum: 60,
			tied: false,
			syms: &self.syms,
		};
		let s = match self.defs.scores.iter().find( |&&(ref k, _)| k == key ) {
			Some( &(_, ref v) ) => v,
			None                => return misc::Error::new( "" ),
		};
		let mut dst = Vec::new();
		self.generate_score( &span, s, &mut dst )?;
		Ok( dst )
	}

	fn generate_score( &self, span: &Span, score: &Box<ast::Score>, dst: &mut Vec<FlatNote> ) -> Result<ratio::Ratio, misc::Error> {
		let end = match **score {
			ast::Score::Score( ref ns ) => {
				let mut nnum = span.nnum;
				for (i, n) in ns.iter().enumerate() {
					let span = Span{
						bgn: span.bgn + i as i64,
						end: span.bgn + i as i64 + 1,
						nnum: nnum,
						tied: false,
						syms: span.syms,
					};
					nnum = self.generate_note( &span, n, dst )?;
				}
				span.bgn + ns.len() as i64
			}
			ast::Score::Variable( ref key ) => {
				let s = match self.defs.scores.iter().find( |&&(ref k, _)| k == key ) {
					Some( &(_, ref v) ) => v,
					None                => return misc::Error::new( "" ),
				};
				self.generate_score( &span, s, dst )?
			},
			ast::Score::With( ref lhs, ref key, ref rhs ) => {
				let mut dst_rhs = Vec::new();
				self.generate_score( &span, rhs, &mut dst_rhs )?;
				let mut syms = span.syms.clone(); // XXX
				syms.insert( *key, dst_rhs );
				let span = Span{
					bgn: span.bgn,
					end: span.end,
					nnum: span.nnum,
					tied: false,
					syms: &syms,
				};
				self.generate_score( &span, lhs, dst )?
			},
			ast::Score::Parallel( ref ss ) => {
				let mut t = span.bgn;
				for s in ss.iter() {
					let span = Span{
						bgn: span.bgn,
						end: span.end,
						nnum: span.nnum,
						tied: false,
						syms: span.syms,
					};
					t = t.max( self.generate_score( &span, s, dst )? );
				}
				t
			},
			ast::Score::Sequence( ref ss ) => {
				let mut t = span.bgn;
				for s in ss.iter() {
					let span = Span{
						bgn: t,
						end: t,
						nnum: span.nnum,
						tied: false,
						syms: span.syms,
					};
					t = self.generate_score( &span, s, dst )?;
				}
				t
			},
		};
		Ok( end )
	}

	fn generate_note( &self, span: &Span, note: &Box<ast::Note>, dst: &mut Vec<FlatNote> ) -> Result<i32, misc::Error> {
		let nnum = match **note {
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
						let nnum = span.nnum / 12 * 12 + (f.nnum + sig) % 12;
						nnum - if nnum <= span.nnum { 0 } else { 12 }
					},
					ast::Dir::Upper => {
						let nnum = span.nnum / 12 * 12 + (f.nnum + sig) % 12;
						nnum + if nnum >= span.nnum { 0 } else { 12 }
					},
				};
				dst.push( FlatNote{
					bgn: span.bgn,
					end: span.end,
					nnum: nnum,
					tied: span.tied,
				} );
				nnum
			},
			ast::Note::Rest => {
				span.nnum
			},
			ast::Note::Repeat => {
				panic!();
			},
			ast::Note::Octave( oct ) => {
				span.nnum + oct * 12
			},
			ast::Note::Chord( ref ns ) => {
				let mut span = Span{
					bgn: span.bgn,
					end: span.end,
					nnum: span.nnum,
					tied: span.tied,
					syms: span.syms,
				};

				let mut it = ns.iter();
				if let Some( n ) = it.next() {
					span.nnum = self.generate_note( &span, n, dst )?;
				}
				let nnum1 = span.nnum;
				for n in it {
					span.nnum = self.generate_note( &span, n, dst )?;
				}
				nnum1
			},
			ast::Note::Group( ref ns ) => {
				let tot: i32 = ns.iter().map( |&(_, i)| i ).sum();
				let mut nnum = span.nnum;
				let mut acc = 0;
				for &(ref n, i) in ns.iter() {
					let span = Span{
						bgn: span.bgn + (span.end - span.bgn) * ratio::Ratio::new( (acc    ) as i64, tot as i64 ),
						end: span.bgn + (span.end - span.bgn) * ratio::Ratio::new( (acc + i) as i64, tot as i64 ),
						nnum: nnum,
						tied: acc == 0 && span.tied,
						syms: span.syms,
					};
					nnum = self.generate_note( &span, n, dst )?;
					acc += i;
				}
				nnum
			},
			ast::Note::Tie( ref n ) => {
				let span = Span{
					bgn: span.bgn,
					end: span.end,
					nnum: span.nnum,
					tied: true,
					syms: span.syms,
				};
				self.generate_note( &span, n, dst )?
			},
		};
		Ok( nnum )
	}
}
