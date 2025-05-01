use std::env;

fn main() {
    let bond_include = env::var("BOND_INCLUDE_DIR").unwrap_or_else(|_| "/usr/local/include".to_string());
    let bond_lib = env::var("BOND_LIB_DIR").unwrap_or_else(|_| "/usr/local/lib".to_string());

    cc::Build::new()
        .cpp(true)
        .file("ffi/bond_ffi.cpp")
        .include("ffi")
        .include(&bond_include)
        .flag("-std=c++14")
        .compile("bond_ffi");

    println!("cargo:rustc-link-search=native={}", bond_lib);
    println!("cargo:rustc-link-lib=bond");
    println!("cargo:rustc-link-lib=boost_system");
}