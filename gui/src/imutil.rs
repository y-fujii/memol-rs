// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use imgui;
use std::*;


pub struct DrawContext<'a> {
	pub draw_list: &'a mut imgui::ImDrawList,
	pub origin: imgui::ImVec2,
	pub size: imgui::ImVec2,
}

impl<'a> DrawContext<'a> {
	pub fn new() -> DrawContext<'static> {
		use imgui::*;
		unsafe {
			DrawContext{
				draw_list: &mut *GetWindowDrawList(),
				origin: GetWindowContentRegionMin(),
				size: GetWindowContentRegionMax() - GetWindowContentRegionMin(),
			}
		}
	}

	pub fn add_line( &mut self, a: imgui::ImVec2, b: imgui::ImVec2, col: u32, thickness: f32 ) {
		unsafe {
			self.draw_list.AddLine( &(self.origin + a), &(self.origin + b), col, thickness );
		}
	}

	pub fn add_rect_filled( &mut self, a: imgui::ImVec2, b: imgui::ImVec2, col: u32, rounding: f32, flags: i32 ) {
		unsafe {
			self.draw_list.AddRectFilled( &(self.origin + a), &(self.origin + b), col, rounding, flags );
		}
	}
}

// XXX
pub fn srgb_gamma( r: f32, g: f32, b: f32, a: f32 ) -> u32 {
	(((r.powf( 1.0 / 2.2 ) * 255.0) as u32) <<  0) |
	(((g.powf( 1.0 / 2.2 ) * 255.0) as u32) <<  8) |
	(((b.powf( 1.0 / 2.2 ) * 255.0) as u32) << 16) |
	(((a                   * 255.0) as u32) << 24)
}

pub fn set_scale( s: f32, font_size: f32, font: &[u8] ) {
	unsafe {
		let io = imgui::get_io();
		let mut cfg = imgui::ImFontConfig::new();
		cfg.FontDataOwnedByAtlas = false;
		(*io.Fonts).AddFontFromMemoryTTF(
			font.as_ptr() as *mut os::raw::c_void,
			font.len() as i32, (font_size * s).round(), &cfg, ptr::null(),
		);

		let style = imgui::get_style();
		style.WindowPadding          = (style.WindowPadding          * s).round();
		style.WindowMinSize          = (style.WindowMinSize          * s).round();
		style.WindowRounding         = (style.WindowRounding         * s).round();
		style.WindowTitleAlign       = (style.WindowTitleAlign       * s).round();
		style.ChildWindowRounding    = (style.ChildWindowRounding    * s).round();
		style.FramePadding           = (style.FramePadding           * s).round();
		style.FrameRounding          = (style.FrameRounding          * s).round();
		style.ItemSpacing            = (style.ItemSpacing            * s).round();
		style.ItemInnerSpacing       = (style.ItemInnerSpacing       * s).round();
		style.TouchExtraPadding      = (style.TouchExtraPadding      * s).round();
		style.IndentSpacing          = (style.IndentSpacing          * s).round();
		style.ColumnsMinSpacing      = (style.ColumnsMinSpacing      * s).round();
		style.ScrollbarSize          = (style.ScrollbarSize          * s).round();
		style.ScrollbarRounding      = (style.ScrollbarRounding      * s).round();
		style.GrabMinSize            = (style.GrabMinSize            * s).round();
		style.GrabRounding           = (style.GrabRounding           * s).round();
		style.ButtonTextAlign        = (style.ButtonTextAlign        * s).round();
		style.DisplayWindowPadding   = (style.DisplayWindowPadding   * s).round();
		style.DisplaySafeAreaPadding = (style.DisplaySafeAreaPadding * s).round();
		style.CurveTessellationTol   = (style.CurveTessellationTol   * s).round();
	}
}

pub fn begin_root( flags: u32 ) {
	use imgui::*;
	unsafe {
		let size = get_io().DisplaySize;
		let rounding = get_style().WindowRounding;
		let padding  = get_style().WindowPadding;
		PushStyleVar( ImGuiStyleVar_WindowRounding as i32, 0.0 );
		PushStyleVar1( ImGuiStyleVar_WindowPadding as i32, &ImVec2::zero() );
		SetNextWindowPos( &ImVec2::zero(), ImGuiSetCond_Always as i32 );
		SetNextWindowSize( &size, ImGuiSetCond_Always as i32 );
		Begin1(
			c_str!( "root" ), &mut true, &size, 0.0,
			(ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoResize |
			 ImGuiWindowFlags_NoBringToFrontOnFocus |
			 ImGuiWindowFlags_NoTitleBar | flags) as i32
		);
		PushStyleVar( ImGuiStyleVar_WindowRounding as i32, rounding );
		PushStyleVar1( ImGuiStyleVar_WindowPadding as i32, &padding );
	}
}

pub fn end_root() {
	use imgui::*;
	unsafe {
		PopStyleVar( 2 );
		End();
		PopStyleVar( 2 );
	}
}
