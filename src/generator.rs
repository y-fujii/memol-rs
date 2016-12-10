// by Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under 2-clause BSD License.
use std::*;
use ratio;
use ast;


#[derive(Debug)]
pub struct Error;

impl fmt::Display for Error {
	fn fmt( &self, f: &mut fmt::Formatter ) -> fmt::Result {
		write!( f, "" )
	}
}

impl error::Error for Error {
	fn description( &self ) -> &str {
		""
	}
}

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

	pub fn generate( &self, key: &str ) -> Result<Vec<FlatNote>, Error> {
		let ninf = ratio::Ratio::new( -1, 0 );
		let pinf = ratio::Ratio::new(  1, 0 );
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
			nnum: 60,
			tied: false,
			syms: &syms,
		};

		let phra = match self.defs.scores.iter().find( |&&(ref k, _)| k == key ) {
			Some( &(_, ref v) ) => v,
			None                => return Err( Error{} ),
		};
		let mut dst = Vec::new();
		self.generate_phrase( &span, phra, &mut dst )?;
		return Ok( dst );
	}

	fn generate_phrase<'b>( &self, span: &Span<'b>, phra: &Box<ast::Phrase>, dst: &mut Vec<FlatNote> ) -> Result<Span<'b>, Error> {
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
					nnum = self.generate_note( &span, n, dst )?;
				}
				return Ok( Span{
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

	fn generate_note( &self, span: &Span, note: &Box<ast::Note>, dst: &mut Vec<FlatNote> ) -> Result<i32, Error> {
		match **note {
			ast::Note::Note( dir, sym, ord, sig ) => {
				let fs = match span.syms.get( &sym ) {
					Some( fs ) => fs,
					None       => return Err( Error{} ),
				};
				let f = match fs.iter().filter( |n| n.bgn <= span.bgn && span.bgn < n.end ).nth( ord as usize ) {
					Some( f ) => f,
					None      => return Err( Error{} ),
				};
				let nnum = match dir {
					ast::Dir::Absolute => f.nnum + sig,
					ast::Dir::Lower => {
						// XXX: negative nnum.
						let nnum = span.nnum / 12 * 12 + (f.nnum + sig) % 12;
						nnum - if nnum <= span.nnum { 0 } else { 12 }
					},
					ast::Dir::Upper => {
						// XXX: negative nnum.
						let nnum = span.nnum / 12 * 12 + (f.nnum + sig) % 12;
						nnum + if nnum >= span.nnum { 0 } else { 12 }
					},
				};
				dst.push( FlatNote{
					bgn: span.bgn,
					end: span.end,
					nnum: nnum,
				} );
				return Ok( nnum );
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
					nnum = self.generate_note( &span, n, dst )?;
					acc += i;
				}
				return Ok( nnum );
			},
			_ => panic!(),
		}
	}
}
