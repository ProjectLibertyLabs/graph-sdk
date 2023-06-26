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
	protobuf_codegen::Codegen::new()
		.protoc()
		.includes(&["protos"])
		.input("protos/input.proto")
		.input("protos/output.proto")
		.out_dir("src/proto_types")
		.run_from_script();

	// generate new Java files
	Command::new("protoc")
		.args([
			"--java_out",
			"../../java/lib/src/main/java/",
			"protos/input.proto",
			"protos/output.proto",
		])
		.spawn()
		.unwrap();
}
