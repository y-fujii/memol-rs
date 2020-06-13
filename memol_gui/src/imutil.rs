// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use crate::imgui::*;
use std::*;

#[macro_export]
macro_rules! c_str {
	($e: tt) => (
		concat!( $e, "\0" ).as_ptr() as *const os::raw::c_char
	);
	($e: tt, $($arg: tt)*) => (
		format!( concat!( $e, "\0" ), $($arg)* ).as_ptr() as *const os::raw::c_char
	)
}

pub struct DrawContext {
    pub draw_list: *mut ImDrawList,
    pub a: ImVec2,
    pub b: ImVec2,
    pub clip_min: ImVec2,
    pub clip_max: ImVec2,
}

impl DrawContext {
    pub fn new(a: f32, b: ImVec2) -> DrawContext {
        unsafe {
            DrawContext {
                draw_list: GetWindowDrawList(),
                a: ImVec2::new(a, -a),
                b: GetWindowPos()
                    + ImVec2::new(GetWindowContentRegionMin().x + b.x, GetWindowContentRegionMax().y - b.y),
                clip_min: GetWindowPos(),
                clip_max: GetWindowPos() + GetWindowSize(),
            }
        }
    }

    pub fn add_line(&mut self, v0: ImVec2, v1: ImVec2, col: u32, thickness: f32) {
        let (lt, rb) = self.to_global_rect(v0, v1);
        if self.intersect_aabb(lt, rb) {
            unsafe {
                (*self.draw_list).AddLine(&lt, &rb, col, self.a.x * thickness);
            }
        }
    }

    pub fn add_rect_filled(&mut self, v0: ImVec2, v1: ImVec2, col: u32, rounding: f32, flags: i32) {
        let (lt, rb) = self.to_global_rect(v0, v1);
        if self.intersect_aabb(lt, rb) {
            unsafe {
                (*self.draw_list).AddRectFilled(&lt, &rb, col, self.a.x * rounding, flags);
            }
        }
    }

    #[allow(dead_code)]
    pub fn add_invisible_button(&mut self, v0: ImVec2, v1: ImVec2, text: &str) -> bool {
        let (lt, rb) = self.to_global_rect(v0, v1);
        unsafe {
            SetCursorScreenPos(&lt);
            InvisibleButton(text.as_ptr() as *const _, &(rb - lt))
        }
    }

    pub fn add_dummy(&mut self, v0: ImVec2, v1: ImVec2) {
        let (lt, rb) = self.to_global_rect(v0, v1);
        unsafe {
            SetCursorScreenPos(&lt);
            Dummy(&(rb - lt));
        }
    }

    pub fn to_global(&self, v: ImVec2) -> ImVec2 {
        self.a * v + self.b
    }

    pub fn to_global_rect(&self, v0: ImVec2, v1: ImVec2) -> (ImVec2, ImVec2) {
        let v0 = self.to_global(v0);
        let v1 = self.to_global(v1);
        let lt = ImVec2::new(f32::min(v0.x, v1.x), f32::min(v0.y, v1.y));
        let rb = ImVec2::new(f32::max(v0.x, v1.x), f32::max(v0.y, v1.y));
        (lt, rb)
    }

    pub fn to_local(&self, v: ImVec2) -> ImVec2 {
        (v - self.b) / self.a
    }

    fn intersect_aabb(&self, v0: ImVec2, v1: ImVec2) -> bool {
        self.clip_min.x <= v1.x && v0.x <= self.clip_max.x && self.clip_min.y <= v1.y && v0.y <= self.clip_max.y
    }
}

pub fn srgb_linear_to_gamma(col: ImVec4) -> ImVec4 {
    let f = |x: f32| {
        if x <= 0.0031308 {
            12.92 * x
        } else {
            1.055 * f32::powf(x, 1.0 / 2.4) - 0.055
        }
    };
    ImVec4::new(f(col.x), f(col.y), f(col.z), col.w)
}

pub fn srgb_gamma_to_linear(col: ImVec4) -> ImVec4 {
    let f = |x: f32| {
        if x <= 0.04045 {
            (1.0 / 12.92) * x
        } else {
            f32::powf((1.0 / 1.055) * (x + 0.055), 2.4)
        }
    };
    ImVec4::new(f(col.x), f(col.y), f(col.z), col.w)
}

pub fn pack_color(col: ImVec4) -> u32 {
    let f = |x: f32| (x * 255.0 + 0.5) as u32;
    f(col.x) | (f(col.y) << 8) | (f(col.z) << 16) | (f(col.w) << 24)
}

pub fn root_begin(name: &str, pos: ImVec2, size: ImVec2, pad: bool, flags: ImGuiWindowFlags_) {
    assert!(name.ends_with('\0'));
    unsafe {
        let rounding = get_style().WindowRounding;
        let border = get_style().WindowBorderSize;
        let padding = get_style().WindowPadding;
        PushStyleVar(ImGuiStyleVar_WindowRounding as i32, 0.0);
        PushStyleVar(ImGuiStyleVar_WindowBorderSize as i32, 0.0);
        PushStyleVar1(
            ImGuiStyleVar_WindowPadding as i32,
            &if pad { padding } else { ImVec2::zero() },
        );
        SetNextWindowPos(&pos, ImGuiCond_Always as i32, &ImVec2::zero());
        SetNextWindowSize(&size, ImGuiCond_Always as i32);
        Begin(
            name.as_ptr() as *const i8,
            ptr::null_mut(),
            ((ImGuiWindowFlags_NoMove
                | ImGuiWindowFlags_NoResize
                | ImGuiWindowFlags_NoBringToFrontOnFocus
                | ImGuiWindowFlags_NoTitleBar)
                ^ flags) as i32,
        );
        PushStyleVar(ImGuiStyleVar_WindowRounding as i32, rounding);
        PushStyleVar(ImGuiStyleVar_WindowBorderSize as i32, border);
        PushStyleVar1(ImGuiStyleVar_WindowPadding as i32, &padding);
    }
}

pub fn root_end() {
    unsafe {
        PopStyleVar(3);
        End();
        PopStyleVar(3);
    }
}

/*
pub fn text_size(text: &str) -> ImVec2 {
    let ptr = text.as_ptr() as *const _;
    unsafe { CalcTextSize(ptr, ptr.add(text.len()), false, -1.0) }
}
*/

pub fn show_text(text: &str) {
    let ptr = text.as_ptr() as *const _;
    unsafe {
        TextUnformatted(ptr, ptr.add(text.len()));
    }
}

pub fn message_dialog(title: &str, text: &str) -> bool {
    unsafe {
        let pos = 0.5 * get_io().DisplaySize;
        SetNextWindowPos(&pos, ImGuiCond_Always as i32, &ImVec2::new(0.5, 0.5));
        Begin(
            c_str!("{}", title),
            ptr::null_mut(),
            (ImGuiWindowFlags_AlwaysAutoResize | ImGuiWindowFlags_NoTitleBar) as i32,
        );
        show_text(text);
        let ok = IsWindowHovered(0) && IsMouseClicked(0, false);
        End();
        ok
    }
}

pub fn set_theme(base: ImVec4, fg: ImVec4, bg: ImVec4) {
    let hovered = (5.0 * base + 1.0 * fg) / 6.0;
    let active = (4.0 * base + 2.0 * fg) / 6.0;

    let style = get_style();
    style.WindowBorderSize = 0.0;
    style.WindowRounding = 0.0;
    style.ChildRounding = 0.0;
    style.PopupRounding = 0.0;
    style.FrameRounding = 0.0;
    style.WindowMinSize = ImVec2::zero();
    style.Colors[ImGuiCol_Text as usize] = fg;
    style.Colors[ImGuiCol_Border as usize] = base;
    style.Colors[ImGuiCol_WindowBg as usize] = bg;
    style.Colors[ImGuiCol_PopupBg as usize] = bg;
    style.Colors[ImGuiCol_ScrollbarBg as usize] = bg;
    style.Colors[ImGuiCol_ScrollbarGrab as usize] = base;
    style.Colors[ImGuiCol_ScrollbarGrabHovered as usize] = hovered;
    style.Colors[ImGuiCol_ScrollbarGrabActive as usize] = active;
    style.Colors[ImGuiCol_Button as usize] = base;
    style.Colors[ImGuiCol_ButtonHovered as usize] = hovered;
    style.Colors[ImGuiCol_ButtonActive as usize] = active;
    style.Colors[ImGuiCol_FrameBg as usize] = base;
    style.Colors[ImGuiCol_FrameBgHovered as usize] = hovered;
    style.Colors[ImGuiCol_FrameBgActive as usize] = active;
    style.Colors[ImGuiCol_CheckMark as usize] = fg;
}
