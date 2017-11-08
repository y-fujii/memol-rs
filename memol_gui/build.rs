extern crate cc;
extern crate bindgen;
use std::*;


fn main() {
	let out_dir = env::var( "OUT_DIR" ).unwrap();

	let file = path::Path::new( &out_dir ).join( "imgui_gen.rs" );
	bindgen::Builder::default()
		.clang_arg( "-x" )
		.clang_arg( "c++" )
		.clang_arg( "-std=c++14" )
		.enable_cxx_namespaces()
		.header( "imgui/imgui.h" )
		.whitelist_function( "Im.*" )
		.whitelist_type( "Im.*" )
		.whitelist_var( "Im.*" )
		.prepend_enum_name( false )
		.generate()
		.unwrap()
		.write_to_file( &file )
		.unwrap();

	cc::Build::new()
		.cpp( true )
		.cpp_link_stdlib( None )
		.flag( "-std=c++14" )
		.flag( "-fno-rtti" )
		.flag( "-fno-exceptions" )
		.flag( "-fno-threadsafe-statics" )
		.flag( "-fno-use-cxa-atexit" )
		.flag( "-Wno-implicit-fallthrough" )
		.define( "NDEBUG", None )
		.define( "IMGUI_DISABLE_OBSOLETE_FUNCTIONS", None )
		.file( "imgui/imgui.cpp" )
		.file( "imgui/imgui_draw.cpp" )
		.compile( "libimgui.a" );

	println!( "cargo:rerun-if-changed=imgui/" );
}
