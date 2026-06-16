fn main() {
    println!("cargo:rerun-if-changed=src/io/app-window.slint");

    // If you add a components folder later, add it here too:
    // println!("cargo:rerun-if-changed=src/io/components");

    slint_build::compile("src/io/app-window.slint").unwrap();
}