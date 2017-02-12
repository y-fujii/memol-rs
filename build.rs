extern crate lalrpop;


fn main() {
	lalrpop::process_root().unwrap();
	println!( "cargo:rerun-if-changed=src/parser.lalrpop" );

	if cfg!( target_family = "windows" ) {
		// libjack.dll is here.
		println!( "cargo:rustc-link-search=native=c:/windows" );
		if cfg!( target_pointer_width = "32" ) {
			println!( "cargo:rustc-link-lib=jack" );
		}
		else if cfg!( target_pointer_width = "64" ) {
			println!( "cargo:rustc-link-lib=jack64" );
		}
		else {
			panic!();
		}
	}
	else {
		println!( "cargo:rustc-link-lib=jack" );
	}
}
