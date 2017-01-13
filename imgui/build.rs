extern crate gcc;
extern crate libbindgen;
use std::*;


fn main() {
	let out_dir = env::var( "OUT_DIR" ).unwrap();

	libbindgen::builder()
		.enable_cxx_namespaces()
		.header( "src/imgui.hpp" )
		.hide_type( "ImGuiTextBuffer" )
		.generate()
		.unwrap()
		.write_to_file( path::Path::new( &out_dir ).join( "imgui_gen.rs" ) )
		.unwrap();

	gcc::Config::new()
		.cpp( true )
		.file( "src/imgui.cpp" )
		.compile( "libimgui.a" );
}
