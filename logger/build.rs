use std::env;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
extern crate vcpkg;

fn main() {
    #[cfg(target_os = "windows")]
    {
        let protopkg = vcpkg::find_package("protobuf").unwrap();
        let protobuf_path = protopkg.link_paths[0].parent().unwrap();

        let protobuf_bin_path = protobuf_path
            .join("tools")
            .join("protobuf")
            .join("protoc.exe")
            .to_str()
            .unwrap()
            .to_string();
        env::set_var("PROTOC", protobuf_bin_path);

        let protobuf_inc_path = protobuf_path
            .join("include")
            .join("google")
            .join("protobuf")
            .to_str()
            .unwrap()
            .to_string();
        env::set_var("PROTOC_INCLUDE", protobuf_inc_path);
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, assume protoc is installed and available in PATH
        env::set_var("PROTOC", "protoc");

        // Set PROTOC_INCLUDE to a default location if needed
        let protobuf_inc_path = PathBuf::from("/usr/include/google/protobuf")
            .to_str()
            .unwrap()
            .to_string();
        env::set_var("PROTOC_INCLUDE", protobuf_inc_path);
    }

    prost_build::compile_protos(&["src/indexlog.proto"], &["src/"]).unwrap();
}