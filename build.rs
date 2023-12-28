extern crate embed_resource;
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.contains("windows") {
        // on windows we will set our game icon as icon for the executable
        embed_resource::compile("build/windows/icon.rc");
    }

    if !target.contains("wasm32") {
        // wasm doesn't support file_watcher, but other platforms do. Might as
        // well enable it since it's normally enabled by default.
        // Kinda hacky: https://stackoverflow.com/a/77379837
        println!("cargo:rustc-cfg=feature=\"bevy/file_watcher\"");
    }
}
