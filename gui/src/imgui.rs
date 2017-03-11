// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![allow( dead_code )]
#![allow( non_camel_case_types )]
#![allow( non_snake_case )]
#![allow( non_upper_case_globals )]
use std::*;

include!( concat!( env!( "OUT_DIR" ), "/imgui_gen.rs" ) );

pub use self::root::*;
pub use self::root::ImGui::*;

impl ImVec2 {
	pub fn new( x: f32, y: f32 ) -> Self {
		ImVec2{ x: x, y: y }
	}

	pub fn zero() -> Self {
		ImVec2{ x: 0.0, y: 0.0 }
	}

	pub fn round( &self ) -> Self {
		ImVec2{ x: self.x.round(), y: self.y.round() }
	}
}

impl ops::Neg for ImVec2 {
	type Output = ImVec2;

	fn neg( self ) -> ImVec2 {
		ImVec2::new( -self.x, -self.y )
	}
}

impl ops::Add<ImVec2> for ImVec2 {
	type Output = ImVec2;

	fn add( self, other: ImVec2 ) -> ImVec2 {
		ImVec2::new( self.x + other.x, self.y + other.y )
	}
}

impl ops::Sub<ImVec2> for ImVec2 {
	type Output = ImVec2;

	fn sub( self, other: ImVec2 ) -> ImVec2 {
		ImVec2::new( self.x - other.x, self.y - other.y )
	}
}

impl ops::Mul<f32> for ImVec2 {
	type Output = ImVec2;

	fn mul( self, other: f32 ) -> ImVec2 {
		ImVec2::new( self.x * other, self.y * other )
	}
}

impl ops::MulAssign<f32> for ImVec2 {
	fn mul_assign( &mut self, other: f32 ) {
		self.x *= other;
		self.y *= other;
	}
}

impl ImVec4 {
	pub fn new( x: f32, y: f32, z: f32, w: f32 ) -> Self {
		ImVec4{ x: x, y: y, z: z, w: w }
	}

	pub fn zero() -> Self {
		ImVec4{ x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
	}
}

pub fn get_io() -> &'static mut ImGuiIO {
	unsafe { &mut *GetIO() }
}

pub fn get_style() -> &'static mut ImGuiStyle {
	unsafe { &mut *GetStyle() }
}

pub fn get_draw_data() -> &'static mut ImDrawData {
	unsafe { &mut *GetDrawData() }
}
