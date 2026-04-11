use std::process::Command;

fn main() {
    // Re-run this script when the commit or tags change.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");
    println!("cargo:rerun-if-changed=.git/packed-refs");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = std::path::Path::new(&out_dir).join("version.txt");
    let version = build_version();
    std::fs::write(dest, version).unwrap();
}

fn build_version() -> String {
    let cargo_version = std::env::var("CARGO_PKG_VERSION").unwrap();
    // If the current commit is exactly on a tag, use the plain semver.
    let on_tag = Command::new("git")
        .args(["describe", "--exact-match", "--tags"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if on_tag {
        return cargo_version.to_string();
    }

    // Otherwise append the short SHA so dev builds are clearly identifiable.
    let sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    match sha {
        Some(sha) => {
            let dirty = Command::new("git")
                .args(["status", "--porcelain"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| !o.stdout.is_empty())
                .unwrap_or(false);

            let suffix = if dirty { "-dirty" } else { "" };
            format!("{cargo_version}-dev+{sha}{suffix}")
        }
        // git not available (e.g. a source tarball release) – fall back gracefully.
        None => cargo_version.to_string(),
    }
}
