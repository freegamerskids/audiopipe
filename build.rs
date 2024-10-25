use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    #[cfg(target_os = "windows")]
    {
        println!("cargo::rerun-if-changed=src/platform/windows/audio_capture/LoopbackCapture.cpp");
        println!("cargo::rerun-if-changed=src/platform/windows/audio_capture/api.cpp");

        let capture_file = PathBuf::from(&base_dir).join("src/platform/windows/audio_capture/LoopbackCapture.cpp");
        let api_file = PathBuf::from(&base_dir).join("src/platform/windows/audio_capture/api.cpp");
        let include_lib = PathBuf::from(&base_dir).join("src/platform/windows/audio_capture/include");
        let wil = PathBuf::from(&base_dir).join("extern/wil/include");

        cc::Build::new()
            .cpp(true)
            .file(capture_file)
            .file(api_file)
            .include(wil)
            .include(include_lib)
            .warnings(false)
            .compile("winloopback");
    }
}