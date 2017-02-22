extern crate lalrpop;
use std::*;


fn main() {
	lalrpop::process_root().unwrap();
	println!( "cargo:rerun-if-changed=src/parser.lalrpop" );

	match env::var( "TARGET" ).unwrap().as_str() {
		"x86_64-pc-windows-gnu" => {
			println!( "cargo:rustc-link-search=native=c:/windows" );
			println!( "cargo:rustc-link-lib=jack64" );
		},
		"i686-pc-windows-gnu" => {
			println!( "cargo:rustc-link-search=native=c:/windows" );
			println!( "cargo:rustc-link-lib=jack" );
		},
		_ => {
			println!( "cargo:rustc-link-lib=jack" );
		}
	}
}
