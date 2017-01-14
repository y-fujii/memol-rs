// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
#![allow( dead_code )]
#![allow( non_camel_case_types )]
#![allow( non_snake_case )]
#![allow( non_upper_case_globals )]
use std::*;

include!( concat!( env!( "OUT_DIR" ), "/imgui_gen.rs" ) );

pub use self::root::*;
pub use self::root::ImGui::*;

pub const WindowFlags_NoTitleBar                : i32 = 1 <<  0;
pub const WindowFlags_NoResize                  : i32 = 1 <<  1;
pub const WindowFlags_NoMove                    : i32 = 1 <<  2;
pub const WindowFlags_NoScrollbar               : i32 = 1 <<  3;
pub const WindowFlags_NoScrollWithMouse         : i32 = 1 <<  4;
pub const WindowFlags_NoCollapse                : i32 = 1 <<  5;
pub const WindowFlags_AlwaysAutoResize          : i32 = 1 <<  6;
pub const WindowFlags_ShowBorders               : i32 = 1 <<  7;
pub const WindowFlags_NoSavedSettings           : i32 = 1 <<  8;
pub const WindowFlags_NoInputs                  : i32 = 1 <<  9;
pub const WindowFlags_MenuBar                   : i32 = 1 << 10;
pub const WindowFlags_HorizontalScrollbar       : i32 = 1 << 11;
pub const WindowFlags_NoFocusOnAppearing        : i32 = 1 << 12;
pub const WindowFlags_NoBringToFrontOnFocus     : i32 = 1 << 13;
pub const WindowFlags_AlwaysVerticalScrollbar   : i32 = 1 << 14;
pub const WindowFlags_AlwaysHorizontalScrollbar : i32 = 1 << 15;
pub const WindowFlags_AlwaysUseWindowPadding    : i32 = 1 << 16;

pub const InputTextFlags_CharsDecimal        : i32 = 1 <<  0;
pub const InputTextFlags_CharsHexadecimal    : i32 = 1 <<  1;
pub const InputTextFlags_CharsUppercase      : i32 = 1 <<  2;
pub const InputTextFlags_CharsNoBlank        : i32 = 1 <<  3;
pub const InputTextFlags_AutoSelectAll       : i32 = 1 <<  4;
pub const InputTextFlags_EnterReturnsTrue    : i32 = 1 <<  5;
pub const InputTextFlags_CallbackCompletion  : i32 = 1 <<  6;
pub const InputTextFlags_CallbackHistory     : i32 = 1 <<  7;
pub const InputTextFlags_CallbackAlways      : i32 = 1 <<  8;
pub const InputTextFlags_CallbackCharFilter  : i32 = 1 <<  9;
pub const InputTextFlags_AllowTabInput       : i32 = 1 << 10;
pub const InputTextFlags_CtrlEnterForNewLine : i32 = 1 << 11;
pub const InputTextFlags_NoHorizontalScroll  : i32 = 1 << 12;
pub const InputTextFlags_AlwaysInsertMode    : i32 = 1 << 13;
pub const InputTextFlags_ReadOnly            : i32 = 1 << 14;
pub const InputTextFlags_Password            : i32 = 1 << 15;

pub const TreeNodeFlags_Selected             : i32 = 1 <<  0;
pub const TreeNodeFlags_Framed               : i32 = 1 <<  1;
pub const TreeNodeFlags_AllowOverlapMode     : i32 = 1 <<  2;
pub const TreeNodeFlags_NoTreePushOnOpen     : i32 = 1 <<  3;
pub const TreeNodeFlags_NoAutoOpenOnLog      : i32 = 1 <<  4;
pub const TreeNodeFlags_DefaultOpen          : i32 = 1 <<  5;
pub const TreeNodeFlags_OpenOnDoubleClick    : i32 = 1 <<  6;
pub const TreeNodeFlags_OpenOnArrow          : i32 = 1 <<  7;
pub const TreeNodeFlags_Leaf                 : i32 = 1 <<  8;
pub const TreeNodeFlags_Bullet               : i32 = 1 <<  9;
//pub const TreeNodeFlags_SpanAllAvailWidth  : i32 = 1 << 10;
//pub const TreeNodeFlags_NoScrollOnOpen     : i32 = 1 << 11;
pub const TreeNodeFlags_CollapsingHeader     : i32 = TreeNodeFlags_Framed | TreeNodeFlags_NoAutoOpenOnLog;

pub const SelectableFlags_DontClosePopups    : i32 = 1 << 0;
pub const SelectableFlags_SpanAllColumns     : i32 = 1 << 1;
pub const SelectableFlags_AllowDoubleClick   : i32 = 1 << 2;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
	Tab,
	LeftArrow,
	RightArrow,
	UpArrow,
	DownArrow,
	PageUp,
	PageDown,
	Home,
	End,
	Delete,
	Backspace,
	Enter,
	Escape,
	A,
	C,
	V,
	X,
	Y,
	Z,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Col {
	Text,
	TextDisabled,
	WindowBg,
	ChildWindowBg,
	PopupBg,
	Border,
	BorderShadow,
	FrameBg,
	FrameBgHovered,
	FrameBgActive,
	TitleBg,
	TitleBgCollapsed,
	TitleBgActive,
	MenuBarBg,
	ScrollbarBg,
	ScrollbarGrab,
	ScrollbarGrabHovered,
	ScrollbarGrabActive,
	ComboBg,
	CheckMark,
	SliderGrab,
	SliderGrabActive,
	Button,
	ButtonHovered,
	ButtonActive,
	Header,
	HeaderHovered,
	HeaderActive,
	Column,
	ColumnHovered,
	ColumnActive,
	ResizeGrip,
	ResizeGripHovered,
	ResizeGripActive,
	CloseButton,
	CloseButtonHovered,
	CloseButtonActive,
	PlotLines,
	PlotLinesHovered,
	PlotHistogram,
	PlotHistogramHovered,
	TextSelectedBg,
	ModalWindowDarkening,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StyleVar {
	Alpha,
	WindowPadding,
	WindowRounding,
	WindowMinSize,
	ChildWindowRounding,
	FramePadding,
	FrameRounding,
	ItemSpacing,
	ItemInnerSpacing,
	IndentSpacing,
	GrabMinSize,
	ButtonTextAlign,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ColorEditMode {
	UserSelect = -2,
	UserSelectShowButton = -1,
	RGB = 0,
	HSV = 1,
	HEX = 2,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseCursor {
	None = -1,
	Arrow = 0,
	TextInput,
	Move,
	ResizeNS,
	ResizeEW,
	ResizeNESW,
	ResizeNWSE,
}

pub const SetCond_Always       : i32 = 1 << 0;
pub const SetCond_Once         : i32 = 1 << 1;
pub const SetCond_FirstUseEver : i32 = 1 << 2;
pub const SetCond_Appearing    : i32 = 1 << 3;

impl ImVec2 {
	pub fn new( x: f32, y: f32 ) -> Self {
		ImVec2{ x: x, y: y }
	}

	pub fn zero() -> Self {
		ImVec2{ x: 0.0, y: 0.0 }
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
