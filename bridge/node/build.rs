fn main() {
	let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not defined");
	match target_os.as_str() {
		"linux" => {
			println!("cargo:rustc-link-arg=-Wl,-export-dynamic");
		},
		"macos" => {
			println!("cargo:rustc-link-arg=-rdynamic");
		},
		_ => {},
	};
}
