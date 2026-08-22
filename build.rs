use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    if !target.starts_with("x86_64-pc-windows") {
        return;
    }

    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=caffeinate.ico");

    // Embed the application icon (Explorer, taskbar, shortcuts) via a Win32
    // resource compiled with windres. Skipped when no windres is available,
    // e.g. native MSVC builds without the Windows SDK rc.exe.
    let candidates = ["x86_64-w64-mingw32-windres", "windres"];
    let windres = match candidates.iter().find(|w| {
        Command::new(w)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }) {
        Some(windres) => *windres,
        None => {
            println!(
                "cargo:warning=windres not found - building without embedded application icon"
            );
            return;
        }
    };

    let obj = PathBuf::from(env::var("OUT_DIR").unwrap_or_default()).join("app.res.o");
    let status = Command::new(windres)
        .args([
            "--input",
            "app.rc",
            "--output",
            obj.to_str().unwrap(),
            "--output-format",
            "coff",
        ])
        .status()
        .expect("failed to run windres");
    if !status.success() {
        panic!("windres failed to compile app.rc");
    }

    println!("cargo:rustc-link-arg={}", obj.display());
}
