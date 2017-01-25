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
		.generate()
		.unwrap()
		.write_to_file( &file )
		.unwrap();

	if cfg!( target_family = "windows" ) {
		// "#define _ 1" in "_mingw_mac.h" causes a compile error.
		use std::*;
		use std::io::prelude::*;
		let mut src = String::new();
		fs::File::open( &file ).unwrap().read_to_string( &mut src ).unwrap();
		let dst = src.replace( "pub const _:", "pub const __an_underscore__:" );
		fs::File::create( &file ).unwrap().write_all( dst.as_bytes() ).unwrap();
	}

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
