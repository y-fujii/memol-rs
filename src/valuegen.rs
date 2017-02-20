// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use ratio;
use ast;


#[derive(Debug)]
pub struct FlatValue {
	pub bgn: ratio::Ratio,
	pub end: ratio::Ratio,
	pub value_bgn: ratio::Ratio,
	pub value_end: ratio::Ratio,
}

#[derive(Debug)]
pub struct Ir {
	pub values: Vec<FlatValue>,
}

#[derive(Debug)]
struct Span {
	bgn: ratio::Ratio,
	end: ratio::Ratio,
}

#[derive(Debug)]
struct State {
}

impl Ir {
	pub fn get_value( &self, t: ratio::Ratio ) -> ratio::Ratio {
		let v = self.values.iter().filter( |v| v.bgn <= t && t < v.end ).next().unwrap();
		v.value_bgn + (v.value_end - v.value_bgn) * (t - v.bgn) / (v.end - v.bgn)
	}
}

#[derive(Debug)]
pub struct Generator<'a> {
	defs: &'a ast::Definition,
}

impl<'a> Generator<'a> {
	pub fn new( defs: &ast::Definition ) -> Generator {
		Generator{ defs: defs }
	}

	pub fn generate( &self, key: &str ) -> Result<Option<Ir>, misc::Error> {
		let span = Span{
			bgn: ratio::Ratio::zero(),
			end: ratio::Ratio::zero(),
		};
		let s = match self.defs.values.get( key ) {
			Some( v ) => v,
			None      => return Ok( None ),
		};
		let mut dst = Ir{
			values: Vec::new(),
		};
		self.generate_value_track( s, &span, &mut dst )?;
		Ok( Some( dst ) )
	}

	fn generate_value_track( &self, track: &ast::Ast<ast::ValueTrack>, span: &Span, dst: &mut Ir ) -> Result<ratio::Ratio, misc::Error> {
		let end = match track.ast {
			ast::ValueTrack::ValueTrack( ref vs ) => {
				let mut state = State{};
				for (i, v) in vs.iter().enumerate() {
					let span = Span{
						bgn: span.bgn + i as i64,
						end: span.bgn + i as i64 + 1,
					};
					self.generate_value( v, &span, &mut state, dst )?;
				}
				span.bgn + vs.len() as i64
			}
			ast::ValueTrack::Symbol( ref key ) => {
				let s = match self.defs.values.get( key ) {
					Some( v ) => v,
					None      => return misc::error( track.bgn, "undefined symbol." ),
				};
				self.generate_value_track( s, &span, dst )?
			},
			ast::ValueTrack::Sequence( ref ss ) => {
				let mut t = span.bgn;
				for s in ss.iter() {
					let span = Span{
						bgn: t,
						end: t,
					};
					t = self.generate_value_track( s, &span, dst )?;
				}
				t
			},
		};
		Ok( end )
	}

	fn generate_value<'b>( &self, value: &'b ast::Ast<ast::Value>, span: &Span, state: &mut State, dst: &mut Ir ) -> Result<(), misc::Error> {
		match value.ast {
			ast::Value::Value( v_bgn, v_end ) => {
				dst.values.push( FlatValue{
					bgn: span.bgn,
					end: span.end,
					value_bgn: v_bgn,
					value_end: v_end,
				} );
			},
			ast::Value::Group( ref vs ) => {
				let tot: i32 = vs.iter().map( |&(_, i)| i ).sum();
				let mut acc = 0;
				for &(ref v, i) in vs.iter() {
					let span = Span{
						bgn: span.bgn + (span.end - span.bgn) * ratio::Ratio::new( (acc    ) as i64, tot as i64 ),
						end: span.bgn + (span.end - span.bgn) * ratio::Ratio::new( (acc + i) as i64, tot as i64 ),
					};
					self.generate_value( v, &span, state, dst )?;
					acc += i;
				}
			},
		};
		Ok( () )
	}
}
