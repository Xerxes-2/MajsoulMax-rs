use std::{env, io::Result, path::PathBuf};

fn main() -> Result<()> {
    prost_build::Config::new()
        .type_attribute(
            "lq.ViewSlot",
            "#[derive(::serde::Serialize, ::serde::Deserialize)]",
        )
        .file_descriptor_set_path(
            PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"))
                .join("liqi_desc.bin"),
        )
        .out_dir("src/proto")
        .compile_protos(&["proto/liqi.proto", "proto/sheets.proto"], &["proto/"])?;

    Ok(())
}
