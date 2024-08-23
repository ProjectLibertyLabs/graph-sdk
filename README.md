<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->

[![Release][release-shield]][release-url]
[![Issues][issues-shield]][issues-url]

**Graph-Sdk** is a utility to read and modify social graphs defined by [DSNP](https://www.dsnp.org) protocol and stored on [Frequency](https://github.com/frequency-chain/frequency) blockchain.

# Overview

This repository contains the following modules
- [Core](core) : `DSNP` compatible social graph implementation in Rust
- [Config](config) : All supported environments and their configuration details
- [Bridge](bridge) : Graph SDK bridges for other languages
  - [jni](bridge/jni) : JNI bridge for JVM languages
  - [ffi](bridge/ffi) : FFI bridge for languages such as C/C++ and Swift
- [Java](java): Java and Android wrappers around graph sdk.

# Build

1. Install Rust using the [official instructions](https://www.rust-lang.org/tools/install).
2. Check out this repository
3. `rust-toolchain.toml` specifies the standard toolchain to use. If you have `rustup` installed, it will automatically install the correct toolchain when you run any cargo command.
4. Running following command will try to build the core library.

    ```sh
    make build
    ```

### Build and test JNI
- To build and install the JNI bridge run

    ```sh
    make build-jni
    ```

-  To test Java and JNI bridge run

    ```sh
    make test-jni
    ```
#### Protobuf code generation
When modifying any of the protobuf definitions, it is important to perform the following steps to ensure proper
serialization and deserialization between JNI and Rust types.
```sh
make build-protos
```
- If protobuf is not installed run
```sh
make install-protos
```
### Build and test FFI
- To build and install the FFI bridge run

    ```sh
    make bindgen
    ```

-  To test FFI bridge run

    ```sh
    make test-ffi
    ```
   - FFI tests have a dependency on `libsodium-dev` and expect it to be installed
    ```sh
     apt-get install -y libsodium-dev
    ```
# Examples
Here are a few examples of how to use this library:
* [Basic flow](docs/examples/basic-flow.md)
* [Rust](docs/examples/rust-examples.md)
* [Java](docs/examples/java-examples.md)

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[issues-shield]: https://img.shields.io/github/issues/ProjectLibertyLabs/graph-sdk.svg?style=for-the-badge
[issues-url]: https://github.com/ProjectLibertyLabs/graph-sdk/issues
[release-shield]: https://img.shields.io/github/v/release/ProjectLibertyLabs/graph-sdk?style=for-the-badge
[release-url]: https://github.com/ProjectLibertyLabs/graph-sdk/releases
