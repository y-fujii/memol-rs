// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/imgui_gen.rs"));

pub use self::root::ImGui::*;
pub use self::root::*;
use std::*;

impl ImVec2 {
    pub fn new(x: f32, y: f32) -> Self {
        ImVec2 { x: x, y: y }
    }

    pub fn zero() -> Self {
        ImVec2 { x: 0.0, y: 0.0 }
    }

    pub fn round(self) -> Self {
        ImVec2 {
            x: self.x.round(),
            y: self.y.round(),
        }
    }
}

impl ops::Neg for ImVec2 {
    type Output = ImVec2;

    fn neg(self) -> ImVec2 {
        ImVec2::new(-self.x, -self.y)
    }
}

impl ops::Add<ImVec2> for ImVec2 {
    type Output = ImVec2;

    fn add(self, other: ImVec2) -> ImVec2 {
        ImVec2::new(self.x + other.x, self.y + other.y)
    }
}

impl ops::Sub<ImVec2> for ImVec2 {
    type Output = ImVec2;

    fn sub(self, other: ImVec2) -> ImVec2 {
        ImVec2::new(self.x - other.x, self.y - other.y)
    }
}

impl ops::Mul<ImVec2> for ImVec2 {
    type Output = ImVec2;

    fn mul(self, other: ImVec2) -> ImVec2 {
        ImVec2::new(self.x * other.x, self.y * other.y)
    }
}

impl ops::Mul<ImVec2> for f32 {
    type Output = ImVec2;

    fn mul(self, other: ImVec2) -> ImVec2 {
        ImVec2::new(self * other.x, self * other.y)
    }
}

impl ops::Div<ImVec2> for ImVec2 {
    type Output = ImVec2;

    fn div(self, other: ImVec2) -> ImVec2 {
        ImVec2::new(self.x / other.x, self.y / other.y)
    }
}

impl ImVec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        ImVec4 { x: x, y: y, z: z, w: w }
    }

    pub fn zero() -> Self {
        ImVec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    pub fn constant(v: f32) -> Self {
        ImVec4 { x: v, y: v, z: v, w: v }
    }

    pub fn map<T: FnMut(f32) -> f32>(self, mut f: T) -> Self {
        ImVec4 {
            x: f(self.x),
            y: f(self.y),
            z: f(self.z),
            w: f(self.w),
        }
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn round(self) -> Self {
        ImVec4 {
            x: self.x.round(),
            y: self.y.round(),
            z: self.z.round(),
            w: self.w.round(),
        }
    }
}

impl ops::Add<ImVec4> for ImVec4 {
    type Output = ImVec4;

    fn add(self, other: ImVec4) -> ImVec4 {
        ImVec4::new(self.x + other.x, self.y + other.y, self.z + other.z, self.w + other.w)
    }
}

impl ops::Div<f32> for ImVec4 {
    type Output = ImVec4;

    fn div(self, other: f32) -> ImVec4 {
        ImVec4::new(self.x / other, self.y / other, self.z / other, self.w / other)
    }
}

impl ops::Mul<ImVec4> for f32 {
    type Output = ImVec4;

    fn mul(self, other: ImVec4) -> ImVec4 {
        ImVec4::new(self * other.x, self * other.y, self * other.z, self * other.w)
    }
}

// XXX: RefCell-like test.
pub fn get_io() -> &'static mut ImGuiIO {
    unsafe { &mut *GetIO() }
}

// XXX: RefCell-like test.
pub fn get_style() -> &'static mut ImGuiStyle {
    unsafe { &mut *GetStyle() }
}
