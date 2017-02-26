// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::*;
use misc;
use ratio;
use ast;


#[derive(Debug)]
pub struct FlatValue {
	pub t0: ratio::Ratio,
	pub t1: ratio::Ratio,
	pub v0: ratio::Ratio,
	pub v1: ratio::Ratio,
}

#[derive(Debug)]
pub struct Ir {
	pub values: Vec<FlatValue>,
}

#[derive(Debug)]
struct Span {
	t0: ratio::Ratio,
	t1: ratio::Ratio,
}

#[derive(Debug)]
struct State {
}

impl Ir {
	pub fn get_value( &self, t: ratio::Ratio ) -> ratio::Ratio {
		let f = self.values.iter().filter( |f| f.t0 <= t && t < f.t1 ).next().unwrap();
		f.v0 + (f.v1 - f.v0) * (t - f.t0) / (f.t1 - f.t0)
	}
}

#[derive(Debug)]
pub struct Generator<'a> {
	defs: &'a ast::Definition<'a>,
}

impl<'a> Generator<'a> {
	pub fn new( defs: &'a ast::Definition<'a> ) -> Generator<'a> {
		Generator{ defs: defs }
	}

	pub fn generate( &self, key: &str ) -> Result<Option<Ir>, misc::Error> {
		let span = Span{
			t0: ratio::Ratio::zero(),
			t1: ratio::Ratio::one(),
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
						t0: span.t0 + (span.t1 - span.t0) * i as i64,
						t1: span.t1 + (span.t1 - span.t0) * i as i64,
						.. *span
					};
					self.generate_value( v, &span, &mut state, dst )?;
				}
				span.t0 + (span.t1 - span.t0) * vs.len() as i64
			}
			ast::ValueTrack::Symbol( ref key ) => {
				let s = match self.defs.values.get( key ) {
					Some( v ) => v,
					None      => return misc::error( track.bgn, "undefined symbol." ),
				};
				self.generate_value_track( s, &span, dst )?
			},
			ast::ValueTrack::Sequence( ref ss ) => {
				let mut t = span.t0;
				for s in ss.iter() {
					let span = Span{
						t0: t,
						t1: t + (span.t1 - span.t0),
						.. *span
					};
					t = self.generate_value_track( s, &span, dst )?;
				}
				t
			},
			ast::ValueTrack::Stretch( ref s, r ) => {
				let span = Span{
					t1: span.t0 + r * (span.t1 - span.t0),
					.. *span
				};
				self.generate_value_track( s, &span, dst )?
			},
		};
		Ok( end )
	}

	fn generate_value( &self, value: &ast::Ast<ast::Value>, span: &Span, state: &mut State, dst: &mut Ir ) -> Result<(), misc::Error> {
		match value.ast {
			ast::Value::Value( v0, v1 ) => {
				dst.values.push( FlatValue{
					t0: span.t0,
					t1: span.t1,
					v0: v0,
					v1: v1,
				} );
			},
			ast::Value::Group( ref vs ) => {
				let tot: i32 = vs.iter().map( |&(_, i)| i ).sum();
				let mut acc = 0;
				for &(ref v, i) in vs.iter() {
					let span = Span{
						t0: span.t0 + (span.t1 - span.t0) * ratio::Ratio::new( (acc    ) as i64, tot as i64 ),
						t1: span.t0 + (span.t1 - span.t0) * ratio::Ratio::new( (acc + i) as i64, tot as i64 ),
					};
					self.generate_value( v, &span, state, dst )?;
					acc += i;
				}
			},
		};
		Ok( () )
	}
}
