fn git_hash() -> String {
    use std::process::Command;

    String::from_utf8_lossy(
        &Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .trim()
    .to_string()
}

fn build_date() -> String {
    use datetime::{LocalDateTime, ISO};

    let now = LocalDateTime::now();
    format!("{}", now.date().iso())
}

fn main() {
    println!("cargo:rustc-env=DEVRC_CORE_BUILD_DATE={}", build_date());
    println!("cargo:rustc-env=DEVRC_CORE_GIT_HASH={}", git_hash());
}
