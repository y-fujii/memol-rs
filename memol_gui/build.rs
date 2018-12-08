extern crate cc;
extern crate bindgen;
use std::*;


fn main() {
	let out_dir = env::var( "OUT_DIR" ).unwrap();

	let file = path::Path::new( &out_dir ).join( "imgui_gen.rs" );
	bindgen::Builder::default()
		.clang_arg( "-x" )
		.clang_arg( "c++" )
		.clang_arg( "-std=gnu++14" )
		.clang_arg( "-fno-rtti" )
		.clang_arg( "-fno-exceptions" )
		.clang_arg( "-fno-threadsafe-statics" )
		.clang_arg( "-fno-use-cxa-atexit" )
		.clang_arg( "-DNDEBUG" )
		.clang_arg( "-DIMGUI_DISABLE_OBSOLETE_FUNCTIONS" )
		.enable_cxx_namespaces()
		.header( "imgui/imgui.h" )
		.whitelist_function( "Im.*" )
		.whitelist_type( "Im.*" )
		.whitelist_var( "Im.*" )
		.blacklist_item( "std.*" )
		.prepend_enum_name( false )
		.generate()
		.unwrap()
		.write_to_file( &file )
		.unwrap();

	cc::Build::new()
		.cpp( true )
		.cpp_link_stdlib( None )
		.flag( "-std=gnu++14" )
		.flag( "-fno-rtti" )
		.flag( "-fno-exceptions" )
		.flag( "-fno-threadsafe-statics" )
		.flag( "-fno-use-cxa-atexit" )
		.define( "NDEBUG", None )
		.define( "IMGUI_DISABLE_OBSOLETE_FUNCTIONS", None )
		.file( "imgui/imgui.cpp" )
		.file( "imgui/imgui_draw.cpp" )
		.file( "imgui/imgui_widgets.cpp" )
		.compile( "libimgui.a" );

	println!( "cargo:rerun-if-changed=imgui/" );
}
