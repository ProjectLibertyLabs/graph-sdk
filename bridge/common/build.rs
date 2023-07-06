use std::{fs, process::Command};

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=protos/input.proto");
	println!("cargo:rerun-if-changed=protos/output.proto");

	// remove old generated files
	let paths = ["src/proto_types", "../../java/lib/src/main/java/io/amplica/graphsdk/models"];
	for path in paths {
		let _ = fs::remove_dir_all(path);
		let _ = fs::create_dir(path);
	}

	// generate new Rust files
	Command::new("protoc")
		.args([
			"--rust_out",
			"src/proto_types",
			"protos/input.proto",
			"protos/output.proto",
			"--experimental_allow_proto3_optional",
		])
		.spawn()
		.unwrap();

	// generate new Java files
	Command::new("protoc")
		.args([
			"--java_out",
			"../../java/lib/src/main/java/",
			"protos/input.proto",
			"protos/output.proto",
			"--experimental_allow_proto3_optional",
		])
		.spawn()
		.unwrap();
}
