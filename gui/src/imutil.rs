// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use imgui;
use std::*;


pub struct DrawContext<'a> {
	pub draw_list: &'a mut imgui::ImDrawList,
	pub min: imgui::ImVec2,
	pub max: imgui::ImVec2,
	pub clip: imgui::ImVec2,
}

impl<'a> DrawContext<'a> {
	pub fn new() -> DrawContext<'static> {
		use imgui::*;
		unsafe {
			DrawContext{
				draw_list: &mut *GetWindowDrawList(),
				min: GetWindowContentRegionMin(),
				max: GetWindowContentRegionMax(),
				clip: GetWindowSize(),
			}
		}
	}

	pub fn add_line( &mut self, a: imgui::ImVec2, b: imgui::ImVec2, col: u32, thickness: f32 ) {
		let a = self.min + a;
		let b = self.min + b;
		if self.intersect_aabb( a, b ) {
			unsafe {
				self.draw_list.AddLine( &a, &b, col, thickness );
			}
		}
	}

	pub fn add_rect_filled( &mut self, a: imgui::ImVec2, b: imgui::ImVec2, col: u32, rounding: f32, flags: i32 ) {
		let a = self.min + a;
		let b = self.min + b;
		if self.intersect_aabb( a, b ) {
			unsafe {
				self.draw_list.AddRectFilled( &a, &b, col, rounding, flags );
			}
		}
	}

	pub fn size( &self ) -> imgui::ImVec2 {
		self.max - self.min
	}

	fn intersect_aabb( &self, a: imgui::ImVec2, b: imgui::ImVec2 ) -> bool {
		0.0 <= b.x && a.x <= self.clip.x && 0.0 <= b.y && a.y <= self.clip.y
	}
}

// XXX
pub fn srgb_gamma( col: imgui::ImVec4 ) -> imgui::ImVec4 {
	imgui::ImVec4::new(
		col.x.powf( 1.0 / 2.2 ),
		col.y.powf( 1.0 / 2.2 ),
		col.z.powf( 1.0 / 2.2 ),
		col.w,
	)
}

pub fn pack_color( col: imgui::ImVec4 ) -> u32 {
	(((col.x * 255.0) as u32) <<  0) |
	(((col.y * 255.0) as u32) <<  8) |
	(((col.z * 255.0) as u32) << 16) |
	(((col.w * 255.0) as u32) << 24)
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
			c_str!( "root" ), ptr::null_mut(), &size, 0.0,
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

pub fn set_theme( base: imgui::ImVec4, fg: imgui::ImVec4, bg: imgui::ImVec4 ) {
	use imgui::*;

	let normal  = srgb_gamma( base );
	let hovered = srgb_gamma( base * 0.8 + fg * 0.2 );
	let active  = srgb_gamma( base * 0.6 + fg * 0.4 );
	let fg      = srgb_gamma( fg );
	let bg      = srgb_gamma( bg );

	let style = get_style();
	style.WindowRounding      = 0.0;
	style.ChildWindowRounding = 0.0;
	style.FrameRounding       = 0.0;
	style.Colors[ImGuiCol_Text                 as usize] = fg;
	style.Colors[ImGuiCol_Border               as usize] = normal;
	style.Colors[ImGuiCol_WindowBg             as usize] = bg;
	style.Colors[ImGuiCol_PopupBg              as usize] = bg;
	style.Colors[ImGuiCol_ScrollbarBg          as usize] = bg;
	style.Colors[ImGuiCol_ScrollbarGrab        as usize] = normal;
	style.Colors[ImGuiCol_ScrollbarGrabHovered as usize] = hovered;
	style.Colors[ImGuiCol_ScrollbarGrabActive  as usize] = active;
	style.Colors[ImGuiCol_Button               as usize] = normal;
	style.Colors[ImGuiCol_ButtonHovered        as usize] = hovered;
	style.Colors[ImGuiCol_ButtonActive         as usize] = active;
	style.Colors[ImGuiCol_FrameBg              as usize] = normal;
	style.Colors[ImGuiCol_FrameBgHovered       as usize] = hovered;
	style.Colors[ImGuiCol_FrameBgActive        as usize] = active;
	style.Colors[ImGuiCol_CheckMark            as usize] = active;
}
