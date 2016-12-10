// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD License.
use std::*;
use ratio;
use ast;


#[derive(Copy, Clone, Debug)]
pub struct FlatNote {
	pub bgn: ratio::Ratio,
	pub end: ratio::Ratio,
	pub nnum: i32,
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
}

impl<'a> Generator<'a> {
	pub fn new( defs: &ast::Definition ) -> Generator {
		Generator{ defs: defs }
	}

	pub fn generate( &self, key: &str ) -> Option<Vec<FlatNote>> {
		let ninf = ratio::Ratio::new( -1000, 1 );
		let pinf = ratio::Ratio::new(  1000, 1 );
		let mut syms = collections::HashMap::new();
		syms.insert( '*', vec![
			FlatNote{ bgn: ninf, end: pinf, nnum:  9 },
			FlatNote{ bgn: ninf, end: pinf, nnum: 11 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  0 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  2 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  4 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  5 },
			FlatNote{ bgn: ninf, end: pinf, nnum:  7 },
		] );
		let span = Span{
			bgn: ratio::Ratio::new( 0, 1 ),
			end: ratio::Ratio::new( 0, 1 ),
			nnum: 0,
			tied: false,
			syms: &syms,
		};

		if let Some( &(_, ref v) ) = self.defs.scores.iter().find( |&&(ref k, _)| k == key ) {
			let mut dst = Vec::new();
			if let Some( _ ) = self.generate_phrase( &span, v, &mut dst ) {
				Some( dst )
			}
			else {
				None
			}
		}
		else {
			None
		}
	}

	fn generate_phrase<'b>( &self, span: &Span<'b>, phra: &Box<ast::Phrase>, dst: &mut Vec<FlatNote> ) -> Option<Span<'b>> {
		match **phra {
			ast::Phrase::Score( ref ns ) => {
				let mut nnum = span.nnum;
				for (i, n) in ns.iter().enumerate() {
					let span = Span{
						bgn: span.bgn + i as i32,
						end: span.bgn + i as i32 + 1,
						nnum: nnum,
						tied: false,
						syms: span.syms,
					};
					if let Some( nn ) = self.generate_note( &span, n, dst ) {
						nnum = nn;
					}
					else {
						return None;
					}
				}
				return Some( Span{
					bgn: span.bgn,
					end: span.bgn + ns.len() as i32,
					nnum: nnum,
					tied: false,
					syms: span.syms,
				} );
			}
			_ => panic!(),
		}
	}

	fn generate_note( &self, span: &Span, note: &Box<ast::Note>, dst: &mut Vec<FlatNote> ) -> Option<i32> {
		match **note {
			ast::Note::AbsoluteNote( sym, ord, sig ) => {
				if let Some( fs ) = span.syms.get( &sym ) {
					if let Some( f ) = fs.iter().filter( |n| n.bgn <= span.bgn && span.bgn < n.end ).nth( ord as usize ) {
						let f = FlatNote{
							bgn: span.bgn,
							end: span.end,
							nnum: f.nnum + sig,
						};
						dst.push( f );
						return Some( f.nnum );
					}
					else {
						return None;
					}
				}
				else {
					return None;
				}
			},
			ast::Note::Group( ref ns ) => {
				let tot: i32 = ns.iter().map( |&(_, i)| i ).sum();
				let mut nnum = span.nnum;
				let mut acc = 0;
				for &(ref n, i) in ns.iter() {
					let span = Span{
						bgn: span.bgn + (span.end - span.bgn) * ratio::Ratio::new( acc,     tot ),
						end: span.bgn + (span.end - span.bgn) * ratio::Ratio::new( acc + i, tot ),
						nnum: nnum,
						tied: acc == 0 && span.tied,
						syms: span.syms,
					};
					if let Some( nn ) = self.generate_note( &span, n, dst ) {
						nnum = nn;
					}
					else {
						return None;
					}
					acc += i;
				}
				return Some( nnum );
			},
			_ => panic!(),
		}
	}
}
