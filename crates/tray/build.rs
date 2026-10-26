//! Build script para embutir recursos nativos do Windows (ícone oficial do PrysmView) no executável.

fn main() {
    #[cfg(windows)]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let res_path = std::path::Path::new(&manifest_dir).join("../../assets/app.res");
        if res_path.exists() {
            println!("cargo:rustc-link-arg={}", res_path.display());
        }
        println!("cargo:rerun-if-changed=../../assets/app.res");
        println!("cargo:rerun-if-changed=../../assets/icon.ico");
    }
}
