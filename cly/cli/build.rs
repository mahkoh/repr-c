// SPDX-License-Identifier: GPL-3.0-or-later
fn main() {
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
