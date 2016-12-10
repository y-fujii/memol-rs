extern crate lalrpop;
extern crate libbindgen;


fn main() {
	lalrpop::process_root().unwrap();

	libbindgen::builder()
		.header( "src/cext.h" )
		.generate()
		.unwrap()
		.write_to_file( "src/cext.rs" )
		.unwrap();

	println!( "cargo:rustc-link-lib=jack" );
}
