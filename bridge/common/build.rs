use std::process::Command;

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=protos/input.proto");
	println!("cargo:rerun-if-changed=protos/output.proto");

	// generate proto files
	Command::new("make")
		.current_dir("../..")
		.args(["build-protos"])
		.spawn()
		.unwrap();
}
