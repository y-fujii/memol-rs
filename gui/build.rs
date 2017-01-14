extern crate gcc;
extern crate libbindgen;
use std::*;


fn main() {
	let out_dir = env::var( "OUT_DIR" ).unwrap();

	libbindgen::builder()
		.clang_arg( "-x" )
		.clang_arg( "c++" )
		.enable_cxx_namespaces()
		.header( "imgui/imgui.h" )
		.hide_type( "ImGuiTextBuffer" )
		.generate()
		.unwrap()
		.write_to_file( path::Path::new( &out_dir ).join( "imgui_gen.rs" ) )
		.unwrap();

	gcc::Config::new()
		.cpp( true )
		.file( "imgui/imgui.cpp" )
		.file( "imgui/imgui_draw.cpp" )
		.compile( "libimgui.a" );
}
