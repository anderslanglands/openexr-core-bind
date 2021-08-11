use std::path::Path;

fn main() {
    let openexr_root = std::env::var("OPENEXR_ROOT").unwrap();
    let openexr_include = Path::new(&openexr_root).join("include");
    let openexr_lib = Path::new(&openexr_root).join("lib");

    let bindings = bindgen::Builder::default()
        .header("src/openexr_wrapper.h")
        .clang_arg(format!("-I{}", openexr_include.display()))
        .clang_arg(format!("-I{}/OpenEXR", openexr_include.display()))
        .allowlist_recursively(false)
        .allowlist_function("exr_.**")
        .allowlist_type("exr_.*")
        .allowlist_type("_priv_exr_.*")
        .allowlist_type("_exr_.*")
        .allowlist_type("transcoding_pipeline_buffer_id")
        .blocklist_type("exr_result_t")
        .blocklist_type("exr_attr_v2i_t")
        .blocklist_type("exr_attr_v2f_t")
        .blocklist_type("exr_attr_v2d_t")
        .blocklist_type("exr_attr_v3i_t")
        .blocklist_type("exr_attr_v3f_t")
        .blocklist_type("exr_attr_v3d_t")
        .constified_enum_module("exr_error_code_t")
        .newtype_enum("exr_default_write_mode")
        .newtype_enum("exr_attr_list_access_mode")
        .newtype_enum("exr_storage_t")
        .newtype_enum("exr_compression_t")
        .newtype_enum("exr_envmap_t")
        .newtype_enum("exr_lineorder_t")
        .newtype_enum("exr_tile_level_mode_t")
        .newtype_enum("exr_tile_round_mode_t")
        .newtype_enum("exr_pixel_type_t")
        .rustfmt_bindings(true)
        .generate()
        .expect("bindgen failed");

    let out_path = Path::new(&std::env::var("OUT_DIR").unwrap())
        .join("openexr_wrapper.rs");

    bindings
        .write_to_file(out_path)
        .expect("Could not write bindings");

    bindings
        .write_to_file("openexr_wrapper.rs")
        .expect("Could not write bindings");

    println!("cargo:rustc-link-search=native={}", openexr_lib.display());
    println!("cargo:rustc-link-lib=dylib=OpenEXRCore-3_1");
}
