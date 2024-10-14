use std::{env, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    let base_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    #[cfg(target_os = "windows")]
    {
        println!("cargo::rerun-if-changed=src/audio_capture/platform/windows/LoopbackCapture.cpp");
        println!("cargo::rerun-if-changed=src/audio_capture/platform/windows/api.cpp");

        let capture_file = PathBuf::from(&base_dir).join("src/audio_capture/platform/windows/LoopbackCapture.cpp");
        let api_file = PathBuf::from(&base_dir).join("src/audio_capture/platform/windows/api.cpp");
        let wil = PathBuf::from(&base_dir).join("src/audio_capture/platform/windows/include");

        cc::Build::new()
            .cpp(true)
            .file(capture_file)
            .file(api_file)
            .include(wil)
            .warnings(false)
            .compile("winloopback");
    }
}