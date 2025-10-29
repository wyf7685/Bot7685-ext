use std::process::Command;

fn main() {
    println!("cargo:rustc-env=BUILD_TIME={}", chrono::Utc::now().to_rfc3339());
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if let Ok(hash) = String::from_utf8(output.stdout) {
            println!("cargo:rustc-env=GIT_COMMIT_HASH={}", hash.trim());
        }
    }
}