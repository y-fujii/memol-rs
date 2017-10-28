// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use imgui::*;
use std::*;


pub struct DrawContext<'a> {
	pub draw_list: &'a mut ImDrawList,
	pub min: ImVec2,
	pub max: ImVec2,
	pub clip_min: ImVec2,
	pub clip_max: ImVec2,
}

impl<'a> DrawContext<'a> {
	pub fn new() -> DrawContext<'static> {
		unsafe {
			let pos = GetWindowPos();
			DrawContext{
				draw_list: &mut *GetWindowDrawList(),
				min: pos + GetWindowContentRegionMin(),
				max: pos + GetWindowContentRegionMax(),
				clip_min: pos,
				clip_max: pos + GetWindowSize(),
			}
		}
	}

	pub fn add_line( &mut self, a: ImVec2, b: ImVec2, col: u32, thickness: f32 ) {
		let a = self.min + a;
		let b = self.min + b;
		if self.intersect_aabb( a, b ) {
			unsafe {
				self.draw_list.AddLine( &a, &b, col, thickness );
			}
		}
	}

	pub fn add_rect_filled( &mut self, a: ImVec2, b: ImVec2, col: u32, rounding: f32, flags: i32 ) {
		let a = self.min + a;
		let b = self.min + b;
		if self.intersect_aabb( a, b ) {
			unsafe {
				self.draw_list.AddRectFilled( &a, &b, col, rounding, flags );
			}
		}
	}

	pub fn size( &self ) -> ImVec2 {
		self.max - self.min
	}

	fn intersect_aabb( &self, a: ImVec2, b: ImVec2 ) -> bool {
		self.clip_min.x <= b.x && a.x <= self.clip_max.x &&
		self.clip_min.y <= b.y && a.y <= self.clip_max.y
	}
}

pub fn srgb_gamma( col: ImVec4 ) -> ImVec4 {
	let f = |x: f32| if x <= 0.0031308 {
		12.92 * x
	} else {
		1.055 * x.powf( 1.0 / 2.4 ) - 0.055
	};
	ImVec4::new( f( col.x ), f( col.y ), f( col.z ), col.w )
}

pub fn pack_color( col: ImVec4 ) -> u32 {
	let f = |x: f32| (x * 255.0 + 0.5) as u32;
	f( col.x ) | (f( col.y ) << 8) | (f( col.z ) << 16) | (f( col.w ) << 24)
}

pub fn set_scale( s: f32 ) {
	let style = get_style();
	style.WindowPadding          = (style.WindowPadding          * s).round();
	style.WindowMinSize          = (style.WindowMinSize          * s).round();
	style.WindowRounding         = (style.WindowRounding         * s).round();
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
	style.DisplayWindowPadding   = (style.DisplayWindowPadding   * s).round();
	style.DisplaySafeAreaPadding = (style.DisplaySafeAreaPadding * s).round();
	style.CurveTessellationTol   = (style.CurveTessellationTol   * s).round();
}

pub fn begin_root( flags: u32 ) {
	unsafe {
		let size = get_io().DisplaySize;
		let rounding = get_style().WindowRounding;
		let padding  = get_style().WindowPadding;
		PushStyleVar( ImGuiStyleVar_WindowRounding as i32, 0.0 );
		PushStyleVar1( ImGuiStyleVar_WindowPadding as i32, &ImVec2::zero() );
		SetNextWindowPos( &ImVec2::zero(), ImGuiSetCond_Always as i32, &ImVec2::zero() );
		SetNextWindowSize( &size, ImGuiSetCond_Always as i32 );
		Begin(
			c_str!( "root" ), ptr::null_mut(),
			(ImGuiWindowFlags_NoMove | ImGuiWindowFlags_NoResize |
			 ImGuiWindowFlags_NoBringToFrontOnFocus |
			 ImGuiWindowFlags_NoTitleBar | flags) as i32
		);
		PushStyleVar( ImGuiStyleVar_WindowRounding as i32, rounding );
		PushStyleVar1( ImGuiStyleVar_WindowPadding as i32, &padding );
	}
}

pub fn end_root() {
	unsafe {
		PopStyleVar( 2 );
		End();
		PopStyleVar( 2 );
	}
}

pub fn show_text( text: &str ) {
	let ptr = text.as_ptr() as *const os::raw::c_char;
	unsafe {
		TextUnformatted( ptr, ptr.offset( text.len() as isize ) );
	}
}

pub fn message_dialog( title: &str, text: &str ) {
	unsafe {
		let pos = get_io().DisplaySize * 0.5;
		SetNextWindowPos( &pos, ImGuiSetCond_Always as i32, &ImVec2::new( 0.5, 0.5 ) );
		Begin(
			c_str!( "{}", title ), ptr::null_mut(),
			(ImGuiWindowFlags_AlwaysAutoResize | ImGuiWindowFlags_NoTitleBar) as i32,
		);
			show_text( text );
		End();
	}
}

pub fn set_theme( base: ImVec4, fg: ImVec4, bg: ImVec4 ) {
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
