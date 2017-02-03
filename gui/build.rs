extern crate gcc;
extern crate bindgen;
use std::*;


fn main() {
	let out_dir = env::var( "OUT_DIR" ).unwrap();

	let file = path::Path::new( &out_dir ).join( "imgui_gen.rs" );
	bindgen::builder()
		.clang_arg( "-x" )
		.clang_arg( "c++" )
		.enable_cxx_namespaces()
		.header( "imgui/imgui.h" )
		.whitelisted_function( "Im.*" )
		.whitelisted_type( "Im.*" )
		.whitelisted_var( "Im.*" )
		.generate()
		.unwrap()
		.write_to_file( &file )
		.unwrap();

	gcc::Config::new()
		.cpp( true )
		.cpp_link_stdlib( None )
		.flag( "-fno-rtti" )
		.flag( "-fno-exceptions" )
		.flag( "-fno-threadsafe-statics" )
		.file( "imgui/imgui.cpp" )
		.file( "imgui/imgui_draw.cpp" )
		.compile( "libimgui.a" );
}
