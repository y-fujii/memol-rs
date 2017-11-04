// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use imgui::*;
use std::*;


pub struct DrawContext<'a> {
	pub draw_list: &'a mut ImDrawList,
	pub a: ImVec2,
	pub b: ImVec2,
	pub clip_min: ImVec2,
	pub clip_max: ImVec2,
}

impl<'a> DrawContext<'a> {
	pub fn new( v0: ImVec2, v1: ImVec2 ) -> DrawContext<'static> {
		unsafe {
			DrawContext{
				draw_list: &mut *GetWindowDrawList(),
				a: ImVec2::new( 1.0, -1.0 ),
				b: ImVec2::new( v0.x, v1.y ),
				clip_min: v0,
				clip_max: v1,
			}
		}
	}

	pub fn set_transform( &mut self, a: f32, b: ImVec2 ) {
		self.a = ImVec2::new( a, -a );
		self.b = ImVec2::new( self.clip_min.x, self.clip_max.y ) + b;
	}

	pub fn add_line( &mut self, v0: ImVec2, v1: ImVec2, col: u32, thickness: f32 ) {
		let (lt, rb) = self.transform_rect( v0, v1 );
		if self.intersect_aabb( lt, rb ) {
			unsafe {
				self.draw_list.AddLine( &lt, &rb, col, self.a.x * thickness );
			}
		}
	}

	pub fn add_rect_filled( &mut self, v0: ImVec2, v1: ImVec2, col: u32, rounding: f32, flags: i32 ) {
		let (lt, rb) = self.transform_rect( v0, v1 );
		if self.intersect_aabb( lt, rb ) {
			unsafe {
				self.draw_list.AddRectFilled( &lt, &rb, col, self.a.x * rounding, flags );
			}
		}
	}

	pub fn transform_loc( &self, v: ImVec2 ) -> ImVec2 {
		self.a * v + self.b
	}

	pub fn transform_rect( &self, v0: ImVec2, v1: ImVec2 ) -> (ImVec2, ImVec2) {
		let v0 = self.transform_loc( v0 );
		let v1 = self.transform_loc( v1 );
		let lt = ImVec2::new( f32::min( v0.x, v1.x ), f32::min( v0.y, v1.y ) );
		let rb = ImVec2::new( f32::max( v0.x, v1.x ), f32::max( v0.y, v1.y ) );
		(lt, rb)
	}

	fn intersect_aabb( &self, v0: ImVec2, v1: ImVec2 ) -> bool {
		self.clip_min.x <= v1.x && v0.x <= self.clip_max.x &&
		self.clip_min.y <= v1.y && v0.y <= self.clip_max.y
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

pub fn root_size() -> ImVec2 {
	get_io().DisplaySize
}

pub fn root_begin( flags: u32 ) {
	unsafe {
		let size = root_size();
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

pub fn root_end() {
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
