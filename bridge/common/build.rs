use std::process::Command;

fn main() {
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed=protos/input.proto");
	println!("cargo:rerun-if-changed=protos/output.proto");

	// generate proto files
	let output = Command::new("make")
		.current_dir("../..")
		.args(["build-protos"])
		.output()
		.expect("failed to execute build-proto");

	println!("status: {}", output.status);
	println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
	println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

	assert!(output.status.success());
}
